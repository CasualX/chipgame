use super::*;

#[derive(Default)]
pub enum TimeState {
	/// Wait for player input to start the game.
	#[default]
	Waiting,
	/// Game is running.
	Running,
	/// Game is over, player won or lost.
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

		self.time += 1;

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
			if self.ents.get(ehandle).map(|ent| ent.remove).unwrap_or(false) {
				if let Some(ent) = self.ents.remove(ehandle) {
					self.qt.remove(ehandle, ent.pos);
					self.events.push(GameEvent::EntityRemoved { entity: ehandle, kind: ent.kind });
				}
			}
		}

		self.input = *input;
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
}

pub(super) fn interact_terrain(s: &mut GameState, ent: &mut Entity) {
	let terrain = s.field.get_terrain(ent.pos);

	if matches!(terrain, Terrain::BearTrap) {
		let trapped = matches!(s.get_trap_state(ent.pos), TrapState::Closed);
		if trapped && !ent.trapped {
			s.events.push(GameEvent::EntityTrapped { entity: ent.handle });
		}
		ent.trapped = trapped;
	}

	if !ent.has_moved {
		return;
	}
	ent.has_moved = false;

	match terrain {
		Terrain::GreenButton => {
			for ptr in s.field.terrain.iter_mut() {
				let terrain = *ptr;
				if matches!(terrain, Terrain::ToggleFloor) {
					*ptr = Terrain::ToggleWall;
				}
				else if matches!(terrain, Terrain::ToggleWall) {
					*ptr = Terrain::ToggleFloor;
				}
			}
			s.events.push(GameEvent::ToggleWalls);
			s.events.push(GameEvent::ButtonPress { pos: ent.pos });
		}
		Terrain::RedButton => {
			// Find the template entity connected to the red button
			let Some(conn) = s.field.find_conn_by_src(ent.pos) else { return };
			let ehandle = s.qt.get(conn.dest)[0];
			let Some(template_ent) = s.ents.get(ehandle) else { return };
			// Raise the event before spawning the new entity
			s.events.push(GameEvent::ButtonPress { pos: ent.pos });
			// Spawn a new entity at the template entity's position
			let args = EntityArgs {
				kind: template_ent.kind,
				pos: template_ent.pos,
				face_dir: template_ent.face_dir,
			};
			let h = s.entity_create(&args);
			// Force the new entity to move out of the spawner
			if let Some(mut ent) = s.ents.take(h) {
				// If the entity movement out of the spawner fails, remove it
				if !try_move(s, &mut ent, args.face_dir.unwrap()) {
					ent.remove = true;
				}
				s.ents.put(ent);
			}
		}
		Terrain::BrownButton => {
			s.events.push(GameEvent::ButtonPress { pos: ent.pos });
		}
		Terrain::BlueButton => {
			for other in s.ents.iter_mut() {
				if matches!(other.kind, EntityKind::Tank) {
					if let Some(face_dir) = other.face_dir {
						other.face_dir = Some(face_dir.turn_around());
					}
				}
			}
			s.events.push(GameEvent::ButtonPress { pos: ent.pos });
		}
		Terrain::Teleport => {

		}
		_ => {}
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
			if ent.hidden != hidden {
				ent.hidden = hidden;
				s.events.push(GameEvent::EntityHidden { entity: ent.handle, hidden });
			}
		}
	}
}
