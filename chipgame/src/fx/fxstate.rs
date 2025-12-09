use super::*;

const SCOUT_INPUT_HOLD: f64 = 1.5 / chipcore::FPS as f64;

#[derive(Clone, Default)]
pub struct FxState {
	pub gs: chipcore::GameState,
	pub camera: PlayCamera,
	pub time: f64,
	pub dt: f64,
	pub random: Random,
	pub render: render::RenderState,
	pub objlookup: HashMap<chipcore::EntityHandle, render::ObjectHandle>,
	pub fire_sprites: HashMap<Vec2i, render::ObjectHandle>,
	pub toggle_walls: HashMap<Vec2i, render::ObjectHandle>,
	pub mirage_walls: HashMap<Vec2i, render::ObjectHandle>,
	pub level_number: i32,
	pub next_level_load: f64,
	pub game_start_time: f64,
	pub game_realtime: f32,
	pub game_over: Option<chipcore::GameOverReason>,
	pub hud_enabled: bool,
	pub is_preview: bool,
	pub darken: bool,
	pub darken_time: f64,
	pub events: Vec<FxEvent>,
	pub replay_inputs: Option<Vec<u8>>,
	pub warps_set: i32,
	pub warps_used: i32,
	pub unpauses: i32,
	pub scout_active: bool,
	pub scout_dir_until: [f64; 4],
	pub scout_speed: f32,
}

impl cvar::IVisit for FxState {
	fn visit(&mut self, f: &mut dyn FnMut(&mut dyn cvar::INode)) {
		self.gs.visit(f);
	}
}

impl FxState {
	/// Creates a new FxState initialized for the given level.
	pub fn new(level_number: i32, level_dto: &chipty::LevelDto, rng_seed: chipcore::RngSeed, tiles: &'static [render::TileGfx]) -> Box<FxState> {
		let mut fx = Box::new(FxState::default());
		fx.level_number = level_number;
		fx.render.tiles = tiles;
		fx.hud_enabled = true;
		fx.darken = true;
		fx.darken_time = -1.0;

		// Parse the level data into the game state
		fx.gs.parse(level_dto, rng_seed);

		// Initialize the render field based on the game state
		fx.render.field.width = fx.gs.field.width;
		fx.render.field.height = fx.gs.field.height;
		fx.render.field.terrain.extend_from_slice(&fx.gs.field.terrain);
		for y in 0..fx.gs.field.height {
			for x in 0..fx.gs.field.width {
				let pos = Vec2(x, y);
				let index = (y * fx.gs.field.width + x) as usize;
				let terrain = fx.gs.field.terrain[index];
				match terrain {
					chipty::Terrain::Fire => handlers::create_fire(&mut fx, pos),
					chipty::Terrain::ToggleFloor => handlers::create_toggle_wall(&mut fx, pos, false),
					chipty::Terrain::ToggleWall => handlers::create_toggle_wall(&mut fx, pos, true),
					chipty::Terrain::HiddenWall => handlers::create_wall_mirage(&mut fx, pos),
					chipty::Terrain::InvisibleWall => handlers::create_wall_mirage(&mut fx, pos),
					_ => {}
				}
			}
		}

		// Sync initial game state
		fx.sync();
		return fx;
	}

	pub fn scout(&mut self) {
		self.gs.ts = chipcore::TimeState::Paused;
		self.events.push(FxEvent::Scout);
		self.hud_enabled = true;
		self.scout_init(true);
	}
	pub fn pause(&mut self) {
		self.gs.ts = chipcore::TimeState::Paused;
		self.events.push(FxEvent::Pause);
		self.hud_enabled = false;
		self.scout_init(false);
	}
	pub fn unpause(&mut self) {
		if matches!(self.gs.ts, chipcore::TimeState::Paused) {
			self.gs.ts = chipcore::TimeState::Running;
			self.events.push(FxEvent::Unpause);
			self.hud_enabled = true;
			self.unpauses += 1;
			self.scout_init(false);

			// Center camera on player again
			self.camera.move_teleport = true;
		}
	}
	pub fn think(&mut self, input: &Input, menu_active: bool) {
		if menu_active {
			self.scout_handle_input(input);
			return;
		}

		if !self.gs.is_game_over() {
			if input.start.is_pressed() {
				self.pause();
			}
			else if input.select.is_pressed() {
				self.scout();
			}
		}

		let player_input = chipcore::Input {
			a: input.a.is_held(),
			b: input.b.is_held(),
			up: input.up.is_held(),
			down: input.down.is_held(),
			left: input.left.is_held(),
			right: input.right.is_held(),
			start: input.start.is_held(),
			select: input.select.is_held(),
		};

		if let Some(_) = &mut self.replay_inputs {
			if player_input.has_directional_input() {
				self.replay_inputs = None;
			}
		}
		let replay_input = self.replay_inputs.as_ref().and_then(|inputs| {
			inputs.get(self.gs.time as usize).cloned().map(chipcore::Input::decode)
		});

		self.gs.tick(&replay_input.unwrap_or(player_input));
		self.sync();

		if self.game_start_time == 0.0 && self.gs.time > 0 {
			self.game_start_time = self.time;
		}

		if self.next_level_load != 0.0 && self.time > self.next_level_load {
			self.next_level_load = 0.0;
			let event = if matches!(self.game_over, Some(chipcore::GameOverReason::LevelComplete)) {
				FxEvent::LevelComplete
			}
			else {
				FxEvent::GameOver
			};
			self.events.push(event);
		}

		// Update invisible walls based on player proximity
		for (&pos, &obj_handle) in &self.mirage_walls {
			let Some(obj) = self.render.objects.get_mut(obj_handle) else { continue };

			let Some(player) = chipcore::ps_nearest_ent(&self.gs, pos) else { continue };
			if player.pos.distance_hat(pos) <= 2 {
				if obj.anim.anims.is_empty() && obj.data.alpha < 1.0 {
					obj.anim.anims.push(render::AnimState::FadeIn(render::FadeIn { atime: 0.0 }));
				}
			}
			else {
				if obj.anim.anims.is_empty() && obj.data.alpha > 0.0 {
					obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
				}
			}
		}
	}
	/// Process game events and update FX state accordingly.
	pub fn sync(&mut self) {
		for ev in &self.gs.events.take() {
			eprintln!("GameEvent: {:?}", ev);
			match ev {
				&chipcore::GameEvent::EntityCreated { entity, kind } => handlers::entity_created(self, entity, kind),
				&chipcore::GameEvent::EntityRemoved { entity, kind } => handlers::entity_removed(self, entity, kind),
				&chipcore::GameEvent::EntityStep { entity } => handlers::entity_step(self, entity),
				&chipcore::GameEvent::EntityTurn { entity } => handlers::entity_face_dir(self, entity),
				&chipcore::GameEvent::EntityHidden { entity, hidden } => handlers::entity_hidden(self, entity, hidden),
				&chipcore::GameEvent::EntityTeleport { entity } => handlers::entity_teleport(self, entity),
				&chipcore::GameEvent::PlayerGameOver { entity, reason } => handlers::player_game_over(self, entity, reason),
				&chipcore::GameEvent::PlayerActivity { entity } => handlers::player_activity(self, entity),
				&chipcore::GameEvent::PlayerPush { entity } => handlers::player_push(self, entity),
				&chipcore::GameEvent::PlayerBump { entity } => handlers::player_push(self, entity),
				&chipcore::GameEvent::LockOpened { pos, key } => handlers::lock_opened(self, pos, key),
				&chipcore::GameEvent::FireHidden { pos, hidden } => handlers::fire_hidden(self, pos, hidden),
				&chipcore::GameEvent::TerrainUpdated { pos, old, new } => handlers::terrain_updated(self, pos, old, new),
				&chipcore::GameEvent::GameOver { reason } => handlers::game_over(self, reason),
				&chipcore::GameEvent::SoundFx { sound } => self.events.push(FxEvent::Sound(sound)),
				&chipcore::GameEvent::BombExplode { pos } => handlers::effect(self, pos, render::EffectType::Sparkles),
				&chipcore::GameEvent::WaterSplash { pos } => handlers::effect(self, pos, render::EffectType::Splash),
				_ => {}
			}
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources, time: f64) {
		let dt = time - self.time;
		self.time = time;
		self.dt = dt;
		let ctx = render::UpdateCtx { time: self.time, dt: self.dt };

		if self.gs.time != 0 {
			self.camera.animate_blend();
		}
		if !matches!(self.gs.ts, chipcore::TimeState::Paused) {
			self.camera.animate_move(ctx.time);
		}
		self.scout_camera(ctx.dt);
		self.camera.animate_position(ctx.dt);
		self.render.update(&ctx);

		let camera = self.camera.setup(resx.viewport.size());
		self.render.draw(g, resx, &camera, time);

		if self.hud_enabled {
			self.render_ui(g, resx, time);
		}
	}

	fn scout_handle_input(&mut self, input: &Input) {
		self.scout_speed = if input.a.is_held() { 5.0 } else { 2.0 };

		let scout_until = self.time + SCOUT_INPUT_HOLD;
		if input.up.is_held() {
			self.scout_dir_until[chipty::Compass::Up as usize] = scout_until;
		}
		if input.right.is_held() {
			self.scout_dir_until[chipty::Compass::Right as usize] = scout_until;
		}
		if input.down.is_held() {
			self.scout_dir_until[chipty::Compass::Down as usize] = scout_until;
		}
		if input.left.is_held() {
			self.scout_dir_until[chipty::Compass::Left as usize] = scout_until;
		}
	}
	fn scout_camera(&mut self, dt: f64) {
		if !self.scout_active || dt <= 0.0 {
			return;
		}
		let now = self.time;
		let north = if now <= self.scout_dir_until[chipty::Compass::Up as usize] { 1 } else { 0 };
		let south = if now <= self.scout_dir_until[chipty::Compass::Down as usize] { 1 } else { 0 };
		let east = if now <= self.scout_dir_until[chipty::Compass::Right as usize] { 1 } else { 0 };
		let west = if now <= self.scout_dir_until[chipty::Compass::Left as usize] { 1 } else { 0 };
		let dir = Vec2(east - west, south - north);
		if dir == Vec2::ZERO {
			return;
		}
		let speed = self.scout_speed;
		if speed <= 0.0 {
			return;
		}
		let pixels_per_second = speed * chipcore::FPS as f32;
		let delta = dir.cast::<f32>().vec3(0.0) * (pixels_per_second * dt as f32);
		self.camera.set_target(self.camera.target + delta);
	}
	fn scout_init(&mut self, scout_active: bool) {
		self.scout_active = scout_active;
		self.scout_dir_until = [0.0; 4];
		self.scout_speed = 0.0;
	}
}
