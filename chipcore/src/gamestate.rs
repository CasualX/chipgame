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
	/// Game over.
	GameOver,
}

/// Game state.
#[derive(Default)]
pub struct GameState {
	pub time: i32,
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
		// Remove entities marked for removal at the start of the tick
		// This allows observers to see entities for the tick they are removed
		for ehandle in self.ents.handles() {
			fn check_remove(ent: &Entity) -> bool {
				ent.flags & EF_REMOVE != 0
			}
			if self.ents.get(ehandle).map(check_remove).unwrap_or(false) {
				self.entity_remove(ehandle);
			}
		}

		// Wait for the player to press any direction input to start the game
		match self.ts {
			TimeState::Paused => return,
			TimeState::Waiting => if !input.has_directional_input() { return },
			TimeState::Running => {},
			TimeState::GameOver => return,
		}
		self.ts = TimeState::Running;

		// Check if the player has run out of time
		if self.field.time_limit > 0 && self.time >= self.field.time_limit * FPS {
			ps_game_over(self, GameOverReason::TimeOut);
			return;
		}

		// Handle player input
		ps_input(self, input);

		// Movement phase
		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				if !matches!(ent.kind, EntityKind::Player) {
					(ent.data.movement_phase)(self, &mut ent);
				}
				self.ents.put(ent);
			}
		}

		// Move the player last
		if let Some(mut ent) = self.ents.take(self.ps.master) {
			(ent.data.movement_phase)(self, &mut ent);
			self.ents.put(ent);
		}

		// Action phase
		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				(ent.data.action_phase)(self, &mut ent);
				self.ents.put(ent);
			}
		}

		// Terrain phase
		let mut its = InteractTerrainState::default();
		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				(ent.data.terrain_phase)(self, &mut ent, &mut its);
				self.ents.put(ent);
			}
		}
		if its.toggle_walls & 1 != 0 {
			self.toggle_walls();
		}
		if its.turn_around_tanks & 1 != 0 {
			self.turn_around_tanks();
		}

		self.input = *input;
		self.time += 1;

		// HACK: Spawn the cloned entities on the 'next' tick
		// Otherwise the clones won't behave correctly...
		// * Collides with the entity triggering the button one tile away
		// * Does not trigger buttons as they are spawned
		self.spawn_clones(&its.spawns);
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
		let Some(pl) = self.ents.get(self.ps.master) else { return false };
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

	/// Trigger game over state.
	pub fn game_over(&mut self, reason: GameOverReason) {
		self.events.fire(GameEvent::GameOver { reason });
		self.ts = TimeState::GameOver;
	}

	/// Returns true if the game is over.
	pub fn is_game_over(&self) -> bool {
		matches!(self.ts, TimeState::GameOver)
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
