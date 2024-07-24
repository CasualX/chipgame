use super::*;

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
	pub qt: QuadTree,
	pub rand: Random,
	pub events: Vec<GameEvent>,
	pub ts: TimeState,
	pub input: Input,
	pub inputs: Vec<u8>,
}

impl GameState {
	pub fn load(&mut self, json: &str) {
		self.time = 0;
		self.ts = TimeState::Waiting;
		self.ps.clear();
		self.input = Input::default();

		let ld: FieldDto = serde_json::from_str(json).unwrap();
		self.field.name = ld.name;
		self.field.hint = ld.hint;
		self.field.password = ld.password;
		self.rand.rng = urandom::rng::Xoshiro256::from_seed(ld.seed);
		self.field.time = ld.time;
		self.field.chips = ld.chips;
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
			for _ in 0..size {
				self.field.terrain.push(Terrain::Floor);
			}
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
			if let Some(ent) = self.ents.get(ehandle) {
				update_hidden_flag(self, ent.pos);
			}
		}

		// Find red buttons and mark the connected entities as templates
		for conn in &self.field.conns {
			let terrain = self.field.get_terrain(conn.src);
			if matches!(terrain, Terrain::RedButton) {
				let template = self.qt.get(conn.dest)[0];
				if let Some(template_ent) = self.ents.get_mut(template) {
					let valid = matches!(template_ent.kind,
						EntityKind::Block | EntityKind::Bug | EntityKind::FireBall | EntityKind::PinkBall | EntityKind::Tank |
						EntityKind::Glider | EntityKind::Teeth | EntityKind::Walker | EntityKind::Blob | EntityKind::Paramecium);
					if valid {
						template_ent.flags |= EF_TEMPLATE;
					}
				}
			}
		}

		let chips = ld.entities.iter().filter(|data| matches!(data.kind, EntityKind::Chip)).count();
		println!("Found {} chips", chips);
	}
}

impl GameState {
	pub fn tick(&mut self, input: &Input) {
		if !match self.ts {
			TimeState::Paused => false,
			TimeState::Waiting => input.any(),
			TimeState::Running => true,
		} {
			return;
		}
		self.ts = TimeState::Running;

		if self.field.time > 0 && self.time >= self.field.time * 60 {
			ps_activity(self, PlayerActivity::OutOfTime);
			self.events.push(GameEvent::GameOver { player: self.ps.ehandle });
			return;
		}

		input.encode(&mut self.inputs);
		ps_update_inbuf(self, input);

		// Let entities think
		let handles = self.ents.handles();
		for ehandle in handles.clone() {
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
		for ehandle in handles.clone() {
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

	fn get_trap_state(&self, pos: Vec2i) -> TrapState {
		if let Some(conn) = self.field.find_conn_by_dest(pos) {
			for ehandle in self.qt.get(conn.src) {
				if self.ents.is_valid(ehandle) {
					return TrapState::Open;
				}
			}
		}
		return TrapState::Closed;
	}

	pub fn is_show_hint(&self) -> bool {
		let Some(pl) = self.ents.get(self.ps.ehandle) else { return false };
		let terrain = self.field.get_terrain(pl.pos);
		matches!(terrain, Terrain::Hint)
	}
}

pub(super) fn interact_terrain(s: &mut GameState, ent: &mut Entity) {
	// Play sound only for player and blocks to avoid a cacophony
	let play_sound = matches!(ent.kind, EntityKind::Player | EntityKind::Block);

	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::BearTrap) {
		let trapped = matches!(s.get_trap_state(ent.pos), TrapState::Closed);
		if trapped && ent.flags & EF_TRAPPED == 0 {
			s.events.push(GameEvent::EntityTrapped { entity: ent.handle });
			// Avoid audio spam when the level is initially loaded
			if s.time != 0 {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::TrapEntered });
			}
		}
		ent.flags = if trapped { ent.flags | EF_TRAPPED } else { ent.flags & !EF_TRAPPED };
	}

	if ent.flags & EF_HAS_MOVED == 0 {
		return;
	}
	ent.flags &= !EF_HAS_MOVED;

	match terrain {
		Terrain::GreenButton => {
			for y in 0..s.field.height {
				for x in 0..s.field.width {
					let terrain = s.field.get_terrain(Vec2i::new(x, y));
					let new = match terrain {
						Terrain::ToggleFloor => Terrain::ToggleWall,
						Terrain::ToggleWall => Terrain::ToggleFloor,
						_ => continue,
					};
					s.field.set_terrain(Vec2i::new(x, y), new);
					s.events.push(GameEvent::TerrainUpdated { pos: Vec2i::new(x, y), old: terrain, new });
				}
			}
			if play_sound {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
			}
		}
		Terrain::RedButton => {
			if play_sound {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
			}
		}
		Terrain::BrownButton => {
			if play_sound {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
			}
		}
		Terrain::BlueButton => {
			for other in s.ents.iter_mut() {
				if matches!(other.kind, EntityKind::Tank) {
					if let Some(face_dir) = other.face_dir {
						other.face_dir = Some(face_dir.turn_around());
						s.events.push(GameEvent::EntityTurn { entity: other.handle });
					}
				}
			}
			// Handle the Tank which triggered the button separately
			// as it has been taken out of the entity list
			if matches!(ent.kind, EntityKind::Tank) {
				if let Some(face_dir) = ent.face_dir {
					ent.face_dir = Some(face_dir.turn_around());
					s.events.push(GameEvent::EntityTurn { entity: ent.handle });
				}
			}
			if play_sound {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
			}
		}
		_ => { }
	}

	let mut from_pos = ent.pos;
	let mut from_terrain = terrain;
	if !play_sound {
		from_pos -= ent.face_dir.map(Compass::to_vec).unwrap_or_default();
		from_terrain = s.field.get_terrain(from_pos);
	}

	// Red button spawns entity when stepping _off_ the button only when triggered by a creature...
	// This 'fixes' level 45 Monster Lab... Hope it doesn't break anything else!
	if matches!(from_terrain, Terrain::RedButton) {
		// Find the template entity connected to the red button
		let Some(conn) = s.field.find_conn_by_src(from_pos) else { return };
		let template = s.qt.get(conn.dest)[0];
		let Some(template_ent) = s.ents.get(template) else { return };
		if template_ent.flags & EF_TEMPLATE == 0 {
			return;
		}
		// Spawn a new entity at the template entity's position
		let args = EntityArgs {
			kind: template_ent.kind,
			pos: template_ent.pos,
			face_dir: template_ent.face_dir,
		};
		let ehandle = s.entity_create(&args);
		// Force the new entity to move out of the spawner
		if let Some(mut ent) = s.ents.take(ehandle) {
			// If the entity movement out of the spawner fails, remove it
			let mut remove = false;
			if !try_move(s, &mut ent, args.face_dir.unwrap()) {
				remove = true;
			}
			s.ents.put(ent);
			// Level 45 here again! The level spams so many entities on a single clone machine!
			// Remove the failed clones to prevent the game from crashing!
			if remove {
				s.entity_remove(ehandle);
			}
		}
	}

	if matches!(ent.kind, EntityKind::Player) {
		let mut from_pos = ent.pos;
		if let Some(step_dir) = ent.step_dir {
			from_pos -= step_dir.to_vec();
		}
		if matches!(s.field.get_terrain(from_pos), Terrain::RecessedWall) {
			s.field.set_terrain(from_pos, Terrain::RaisedWall);
			s.events.push(GameEvent::TerrainUpdated { pos: from_pos, old: Terrain::RecessedWall, new: Terrain::RaisedWall });
			s.events.push(GameEvent::SoundFx { sound: SoundFx::WallPopup });
		}
	}
}

pub(super) fn update_hidden_flag(s: &mut GameState, pos: Vec2i) {
	// Hide all template entities on clone machines
	let hide_all = matches!(s.field.get_terrain(pos), Terrain::CloneMachine);

	let mut hidden = hide_all;
	if !hidden {
		for ehandle in s.qt.get(pos) {
			if let Some(ent) = s.ents.get(ehandle) {
				if matches!(ent.kind, EntityKind::Block) {
					hidden = true;
					break;
				}
			}
		}
	}

	for ehandle in s.qt.get(pos) {
		if let Some(ent) = s.ents.get_mut(ehandle) {
			if !hide_all && matches!(ent.kind, EntityKind::Block) {
				continue;
			}
			if (ent.flags & EF_HIDDEN != 0) != hidden {
				ent.flags = if hidden { ent.flags | EF_HIDDEN } else { ent.flags & !EF_HIDDEN };
				s.events.push(GameEvent::EntityHidden { entity: ent.handle, hidden });
			}
		}
	}
}
