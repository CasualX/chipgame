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
	pub ts: TimeState,
	pub ps: PlayerState,
	pub field: Field,
	pub ents: EntityMap,
	pub qt: QuadTree,
	pub rand: Random,
	pub events: Events,
	pub input: Input,
	pub inputs: Vec<u8>,
}

impl GameState {
	pub fn parse(&mut self, ld: &chipty::LevelDto, rng_seed: RngSeed) {
		let seed = match rng_seed {
			RngSeed::Manual(seed) => seed,
			RngSeed::System => {
				let mut seed = [0u8; 8];
				urandom::rng::getentropy(&mut seed);
				u64::from_le_bytes(seed)
			},
		};

		self.time = 0;
		self.ts = TimeState::Waiting;
		self.ps = PlayerState::default();
		self.field.parse(ld, seed);
		self.ents.clear();
		self.qt.init(ld.map.width, ld.map.height);
		self.rand.reseed(seed);
		self.events.clear();
		self.input = Input::default();
		self.inputs.clear();

		// Create entities
		for data in &ld.entities {
			self.entity_create(data);
		}
		// And update their hidden flags
		for ehandle in self.ents.handles() {
			if let Some(ent) = self.ents.take(ehandle) {
				if matches!(ent.kind, EntityKind::Block | EntityKind::IceBlock) {
					self.update_hidden_flag(ent.pos, true);
				}
				self.ents.put(ent);
			}
		}
	}
}

impl GameState {
	/// Advance the game state by one tick.
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
		let mut it_state = InteractTerrainState::default();
		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				it_state.check(self, &mut ent);
				self.ents.put(ent);
			}
		}
		if it_state.toggle_walls & 1 != 0 {
			self.toggle_walls();
		}
		if it_state.turn_around_tanks & 1 != 0 {
			self.turn_around_tanks();
		}

		// Remove entities marked for removal
		for ehandle in self.ents.handles() {
			if self.ents.get(ehandle).map(|ent| ent.flags & EF_REMOVE != 0).unwrap_or(false) {
				self.entity_remove(ehandle);
			}
		}

		self.input = *input;
		self.time += 1;

		// HACK: Spawn the cloned entities on the 'next' tick
		// Otherwise the clones won't move out of the spawner correctly
		// Try it yourself: move this code above the increment of time
		self.spawn_clones(&it_state.spawns);
	}

	/// Returns the trap state at the given position.
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

	/// Spawn cloned entities from spawners.
	pub fn spawn_clones(&mut self, spawns: &[EntityArgs]) {
		for args in spawns {
			// Clones are forced out of the spawner, so they must have a direction
			let Some(face_dir) = args.face_dir else { continue };

			let ehandle = self.entity_create(args);
			if let Some(mut ent) = self.ents.take(ehandle) {
				// Force the new entity to move out of the spawner
				let success = try_move(self, &mut ent, face_dir);
				self.ents.put(ent);
				// If the entity movement out of the spawner fails, remove it
				// This indicates that there's a lot of entities being spawned
				if !success {
					self.entity_remove(ehandle);
				}
			}
		}
	}

	/// Sets terrain at position and fires event if changed.
	pub fn set_terrain(&mut self, pos: Vec2i, terrain: Terrain) {
		if let Some(old) = self.field.set_terrain(pos, terrain) {
			self.events.fire(GameEvent::TerrainUpdated { pos, old, new: terrain });
			// TODO: Update hidden flags when terrain changes to fire?
		}
	}

	/// Returns true if the player is standing on a hint tile.
	pub fn is_show_hint(&self) -> bool {
		let Some(pl) = self.ents.get(self.ps.ehandle) else { return false };
		let terrain = self.field.get_terrain(pl.pos);
		matches!(terrain, Terrain::Hint)
	}

	pub(super) fn update_hidden_flag(&mut self, pos: Vec2i, hidden: bool) {
		for ehandle in self.qt.get(pos) {
			if let Some(ent) = self.ents.get_mut(ehandle) {
				if matches!(ent.kind, EntityKind::Block | EntityKind::Bomb) {
					continue;
				}
				if (ent.flags & EF_HIDDEN != 0) != hidden {
					ent.flags = if hidden { ent.flags | EF_HIDDEN } else { ent.flags & !EF_HIDDEN };
					self.events.fire(GameEvent::EntityHidden { entity: ent.handle, hidden });
				}
			}
		}

		let terrain = self.field.get_terrain(pos);
		if matches!(terrain, Terrain::Fire) {
			self.events.fire(GameEvent::FireHidden { pos, hidden });
		}
	}

	/// Toggle all toggleable walls and floors on the field.
	pub fn toggle_walls(&mut self) {
		for y in 0..self.field.height {
			for x in 0..self.field.width {
				let terrain = self.field.get_terrain(Vec2i::new(x, y));
				let new = match terrain {
					Terrain::ToggleFloor => Terrain::ToggleWall,
					Terrain::ToggleWall => Terrain::ToggleFloor,
					_ => continue,
				};
				self.set_terrain(Vec2i::new(x, y), new);
			}
		}
	}

	/// Turn around all tanks on the field.
	pub fn turn_around_tanks(&mut self) {
		for other in self.ents.iter_mut() {
			if matches!(other.kind, EntityKind::Tank) {
				// Ignore Tank template entities
				if other.flags & EF_TEMPLATE != 0 {
					continue;
				}
				if let Some(face_dir) = other.face_dir {
					other.face_dir = Some(face_dir.turn_around());
					self.events.fire(GameEvent::EntityTurn { entity: other.handle });
				}
			}
		}
	}

	/// Save replay data.
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
