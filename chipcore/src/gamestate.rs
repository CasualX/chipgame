use super::*;

/// Time state.
#[derive(Default)]
pub enum TimeState {
	/// Wait for player input to start the game.
	#[default]
	Waiting,
	/// Game is running.
	Running,
	/// Game is paused.
	Paused,
}

/// Game state.
#[derive(Default)]
pub struct GameState {
	pub time: Time,
	pub ps: PlayerState,
	pub field: Field,
	pub ents: EntityMap,
	pub spawns: Vec<EntityArgs>,
	pub qt: QuadTree,
	pub rand: Random,
	pub events: Events,
	pub ts: TimeState,
	pub input: Input,
	pub inputs: Vec<u8>,
}

impl GameState {
	pub fn parse(&mut self, json: &str, rng_seed: RngSeed) {
		self.time = 0;
		self.ts = TimeState::Waiting;
		self.ps = PlayerState::default();
		self.input = Input::default();

		let ld: LevelDto = serde_json::from_str(json).unwrap();
		self.field.name = ld.name;
		self.field.author = ld.author;
		self.field.hint = ld.hint;
		self.field.password = ld.password;
		self.field.seed = match rng_seed {
			RngSeed::Manual(seed) => seed,
			RngSeed::System => {
				let mut seed = [0u8; 8];
				urandom::rng::getentropy(&mut seed);
				u64::from_le_bytes(seed)
			},
		};
		self.rand.reseed(self.field.seed);
		self.field.time_limit = ld.time_limit;
		self.field.required_chips = ld.required_chips;
		self.field.width = ld.map.width;
		self.field.height = ld.map.height;
		self.field.terrain.clear();
		self.field.conns = ld.connections;

		self.qt.init(ld.map.width, ld.map.height);
		self.ents.clear();

		self.inputs.clear();

		assert!(ld.map.width > 0, "Invalid map width");
		assert!(ld.map.height > 0, "Invalid map height");
		let size = ld.map.width as usize * ld.map.height as usize;
		self.field.terrain.reserve_exact(size);

		if ld.map.data.is_empty() {
			self.field.terrain.resize(size, Terrain::Floor);
		}
		else {
			assert_eq!(ld.map.data.len(), size, "Invalid map data length");
			for y in 0..ld.map.height {
				for x in 0..ld.map.width {
					let index = (y * ld.map.width + x) as usize;
					let terrain = ld.map.legend[ld.map.data[index] as usize];
					self.field.terrain.push(terrain);
				}
			}
		}

		for data in &ld.entities {
			self.entity_create(data);
		}

		for ehandle in self.ents.handles() {
			if let Some(ent) = self.ents.take(ehandle) {
				if matches!(ent.kind, EntityKind::Block | EntityKind::IceBlock) {
					self.update_hidden_flag(ent.pos, true);
				}
				self.ents.put(ent);
			}
		}

		// Find red buttons and mark the connected entities as templates
		for conn in &self.field.conns {
			let terrain = self.field.get_terrain(conn.src);
			if matches!(terrain, Terrain::RedButton) {
				let template = self.qt.get(conn.dest)[0];
				if let Some(template_ent) = self.ents.get_mut(template) {
					let valid = matches!(template_ent.kind,
						EntityKind::Block | EntityKind::IceBlock | EntityKind::Bug | EntityKind::FireBall | EntityKind::PinkBall | EntityKind::Tank |
						EntityKind::Glider | EntityKind::Teeth | EntityKind::Walker | EntityKind::Blob | EntityKind::Paramecium);
					if valid {
						template_ent.flags |= EF_TEMPLATE;
					}
				}
			}
		}

		// let chips = ld.entities.iter().filter(|data| matches!(data.kind, EntityKind::Chip)).count();
		// eprintln!("Found {} chips", chips);
	}
}

impl GameState {
	pub fn tick(&mut self, input: &Input) {
		// Wait for the player to press any direction key to start the game
		if !match self.ts {
			TimeState::Paused => false,
			TimeState::Waiting => input.any_arrows(),
			TimeState::Running => true,
		} {
			return;
		}
		self.ts = TimeState::Running;

		// Check if the player has run out of time
		if self.field.time_limit > 0 && self.time >= self.field.time_limit * 60 {
			ps_activity(self, PlayerActivity::OutOfTime);
			self.events.fire(GameEvent::GameOver { player: () });
			return;
		}

		// Handle player input
		input.encode(&mut self.inputs);
		ps_input(self, input);

		// Spawn the cloned entities
		self.spawn_clones();

		// Let entities think
		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				if !matches!(ent.kind, EntityKind::Player) {
					(ent.data.think)(self, &mut ent);
				}
				self.ents.put(ent);
			}
		}

		// Simulate the player last
		if let Some(mut ent) = self.ents.take(self.ps.ehandle) {
			(ent.data.think)(self, &mut ent);
			self.ents.put(ent);
		}

		// Handle entity-terrain interactions
		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				interact_terrain(self, &mut ent);
				self.ents.put(ent);
			}
		}

		// Remove entities marked for removal
		for ehandle in self.ents.handles() {
			if self.ents.get(ehandle).map(|ent| ent.flags & EF_REMOVE != 0).unwrap_or(false) {
				self.entity_remove(ehandle);
			}
		}

		self.input = *input;
		self.time += 1;
	}

	pub fn get_trap_state(&self, pos: Vec2i) -> TrapState {
		let mut state = TrapState::Closed;
		for conn in &self.field.conns {
			if conn.dest == pos {
				for ehandle in self.qt.get(conn.src) {
					if self.ents.is_valid(ehandle) {
						state = TrapState::Open;
					}
				}
			}
		}
		return state;
	}

	fn spawn_clones(&mut self) {
		let s = self;
		for i in 0..s.spawns.len() {
			let args = &{s.spawns[i]};

			// Clones are forced out of the spawner, so they must have a direction
			let Some(face_dir) = args.face_dir else { continue };

			let ehandle = s.entity_create(args);

			if let Some(mut ent) = s.ents.take(ehandle) {
				let mut remove = false;
				// Force the new entity to move out of the spawner
				if !try_move(s, &mut ent, face_dir) {
					remove = true;
				}
				s.ents.put(ent);
				// If the entity movement out of the spawner fails, remove it
				// This indicates that there's a lot of entities being spawned
				if remove {
					s.entity_remove(ehandle);
				}
			}
		}

		// Clear the spawn list
		s.spawns.clear();
	}

	pub fn set_terrain(&mut self, pos: Vec2i, terrain: Terrain) {
		if let Some(old) = self.field.set_terrain(pos, terrain) {
			self.events.fire(GameEvent::TerrainUpdated { pos, old, new: terrain });
		}
	}

	pub fn is_show_hint(&self) -> bool {
		let Some(pl) = self.ents.get(self.ps.ehandle) else { return false };
		let terrain = self.field.get_terrain(pl.pos);
		matches!(terrain, Terrain::Hint)
	}

	pub(super) fn update_hidden_flag(&mut self, pos: Vec2i, hidden: bool) {
		let s = self;

		for ehandle in s.qt.get(pos) {
			if let Some(ent) = s.ents.get_mut(ehandle) {
				if matches!(ent.kind, EntityKind::Block | EntityKind::Bomb) {
					continue;
				}
				if (ent.flags & EF_HIDDEN != 0) != hidden {
					ent.flags = if hidden { ent.flags | EF_HIDDEN } else { ent.flags & !EF_HIDDEN };
					s.events.fire(GameEvent::EntityHidden { entity: ent.handle, hidden });
				}
			}
		}

		let terrain = s.field.get_terrain(pos);
		if matches!(terrain, Terrain::Fire) {
			s.events.fire(GameEvent::FireHidden { pos, hidden });
		}
	}

	pub fn save_replay(&self, realtime: f32) -> chipty::ReplayDto {
		chipty::ReplayDto {
			date: None,
			ticks: self.time,
			realtime,
			steps: self.ps.steps,
			bonks: self.ps.bonks,
			seed: format!("{:016x}", self.field.seed),
			replay: chipty::encode(&self.inputs),
		}
	}
}
