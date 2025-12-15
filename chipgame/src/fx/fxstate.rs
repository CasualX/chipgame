use super::*;

const SCOUT_INPUT_HOLD: f64 = 1.5 / chipcore::FPS as f64;

#[derive(Clone, Default)]
pub struct FxState {
	pub game: chipcore::GameState,
	pub camera: PlayCamera,
	pub time: f64,
	pub dt: f64,
	pub random: Random,
	pub render: render::RenderState,

	pub game_objects: HashMap<chipcore::EntityHandle, render::ObjectHandle>,
	pub hidden_objects: HashMap<chipcore::EntityHandle, render::ObjectHandle>,
	pub fire_sprites: HashMap<Vec2i, render::ObjectHandle>,
	pub hidden_fire: HashMap<Vec2i, render::ObjectHandle>,
	pub toggle_walls: HashMap<Vec2i, render::ObjectHandle>,
	pub mirage_walls: HashMap<Vec2i, render::ObjectHandle>,
	pub fake_blue_walls: HashMap<Vec2i, render::ObjectHandle>,
	pub water_hazards: HashMap<Vec2i, render::ObjectHandle>,

	pub level_number: i32,
	pub next_level_load: f64,
	pub game_start_time: f64,
	pub game_realtime: f32,
	pub game_over: Option<chipcore::GameOverReason>,
	pub pause_pressed: bool,
	pub hud_enabled: bool,
	pub is_preview: bool,
	pub step_mode: bool,
	pub assist_dist: i32,
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
		self.game.visit(f);
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
		fx.assist_dist = 2;

		// Parse the level data into the game state
		fx.game.parse(level_dto, rng_seed);

		// Initialize the render field based on the game state
		fx.render.field.width = fx.game.field.width;
		fx.render.field.height = fx.game.field.height;
		fx.render.field.terrain.extend_from_slice(&fx.game.field.terrain);
		for y in 0..fx.game.field.height {
			for x in 0..fx.game.field.width {
				let pos = Vec2(x, y);
				let index = (y * fx.game.field.width + x) as usize;
				let terrain = fx.game.field.terrain[index];
				match terrain {
					chipty::Terrain::Fire => handlers::create_fire(&mut fx, pos),
					chipty::Terrain::ToggleFloor => handlers::create_toggle_wall(&mut fx, pos, false),
					chipty::Terrain::ToggleWall => handlers::create_toggle_wall(&mut fx, pos, true),
					chipty::Terrain::HiddenWall => handlers::create_wall_mirage(&mut fx, pos),
					chipty::Terrain::InvisibleWall => handlers::create_wall_mirage(&mut fx, pos),
					chipty::Terrain::WaterHazard => handlers::create_water_hazard(&mut fx, pos),
					chipty::Terrain::FakeBlueWall => handlers::create_fake_blue_wall(&mut fx, pos),
					_ => {}
				}
			}
		}

		// Sync initial game state
		fx.sync();
		return fx;
	}

	pub fn scout(&mut self) {
		self.game.time_state = chipcore::TimeState::Paused;
		self.events.push(FxEvent::ScoutMode);
		self.hud_enabled = true;
		self.scout_init(true);
	}
	pub fn pause(&mut self) {
		self.game.time_state = chipcore::TimeState::Paused;
		self.events.push(FxEvent::PauseGame);
		self.hud_enabled = false;
		self.scout_init(false);
	}
	pub fn resume(&mut self) {
		if matches!(self.game.time_state, chipcore::TimeState::Paused) {
			self.game.time_state = chipcore::TimeState::Running;
			self.events.push(FxEvent::ResumePlay);
			self.hud_enabled = true;
			self.unpauses += 1;
			self.scout_init(false);

			// Center camera on player again
			self.camera.move_teleport = true;
		}
	}
	pub fn think(&mut self, input: &chipcore::Input, menu_active: bool) {
		if menu_active {
			self.scout_handle_input(input);
			return;
		}

		if !self.game.is_game_over() {
			if input.start && !self.pause_pressed {
				self.pause();
			}
			else if input.select && !self.pause_pressed {
				self.scout();
			}
			self.pause_pressed = input.start || input.select;
		}

		let player_input = *input;

		if let Some(_) = &mut self.replay_inputs {
			if player_input.has_directional_input() {
				self.replay_inputs = None;
			}
		}
		let replay_input = self.replay_inputs.as_ref().and_then(|inputs| {
			inputs.get(self.game.time as usize).cloned().map(chipcore::Input::decode)
		});

		let input = replay_input.unwrap_or(player_input);

		// Handle step mode
		let mut run_tick = true;
		if self.step_mode {
			run_tick = self.game.should_tick_step_mode(&input);
			// Reset input buffering to avoid stale inputs
			if !run_tick {
				self.game.input_reset();
			}
		}

		if run_tick {
			self.game.tick(&input);
			self.sync();
		}

		if self.game_start_time == 0.0 && self.game.time > 0 {
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

		let fade_invisible = FadeValues {
			near_alpha: 0.25,
			far_alpha: 0.0,
		};
		let fade_visible = FadeValues {
			near_alpha: 0.75,
			far_alpha: 1.0,
		};

		// Update invisible walls based on player proximity
		fade_assist_dist(&self.mirage_walls, &mut self.render.objects, self.assist_dist, &fade_invisible, |&pos| chipcore::ps_nearest_ent(&self.game, pos).map(|player| player.pos.distance_hat(pos)));
		fade_assist_dist(&self.hidden_fire, &mut self.render.objects, self.assist_dist, &fade_invisible, |&pos| chipcore::ps_nearest_ent(&self.game, pos).map(|player| player.pos.distance_hat(pos)));
		fade_assist_dist(&self.fake_blue_walls, &mut self.render.objects, self.assist_dist, &fade_visible, |&pos| chipcore::ps_nearest_ent(&self.game, pos).map(|player| player.pos.distance_hat(pos)));
		fade_assist_dist(&self.hidden_objects, &mut self.render.objects, self.assist_dist, &fade_invisible, |&ehandle| {
			let ent = self.game.ents.get(ehandle)?;
			let player = chipcore::ps_nearest_ent(&self.game, ent.pos)?;
			let dist = player.pos.distance_hat(ent.pos);
			Some(dist)
		});
	}
	/// Process game events and update FX state accordingly.
	pub fn sync(&mut self) {
		for ev in &self.game.events.take() {
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
				&chipcore::GameEvent::SoundFx { sound } => self.events.push(FxEvent::PlaySound(sound)),
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

		if self.game.time != 0 {
			self.camera.animate_blend();
		}
		if !matches!(self.game.time_state, chipcore::TimeState::Paused) {
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

	fn scout_handle_input(&mut self, input: &chipcore::Input) {
		self.scout_speed = if input.a { 5.0 } else { 2.0 };

		let scout_until = self.time + SCOUT_INPUT_HOLD;
		if input.up {
			self.scout_dir_until[chipty::Compass::Up as usize] = scout_until;
		}
		if input.right {
			self.scout_dir_until[chipty::Compass::Right as usize] = scout_until;
		}
		if input.down {
			self.scout_dir_until[chipty::Compass::Down as usize] = scout_until;
		}
		if input.left {
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

struct FadeValues {
	near_alpha: f32,
	far_alpha: f32,
}

fn fade_assist_dist<T, F: Fn(&T) -> Option<i32>>(
	map: &HashMap<T, render::ObjectHandle>,
	objects: &mut render::ObjectMap,
	fade_dist: i32,
	values: &FadeValues,
	f: F,
) {
	for (key, &obj_handle) in map {
		let Some(obj) = objects.get_mut(obj_handle) else { continue };
		let Some(dist) = f(key) else { continue };
		if dist <= fade_dist {
			fade_to(obj, values.near_alpha, 4.0);
		}
		else {
			fade_to(obj, values.far_alpha, 4.0);
		}
	}
}

fn fade_to(obj: &mut render::Object, target_alpha: f32, fade_spd: f32) {
	for anim in &mut obj.anim.anims {
		if let render::AnimState::FadeTo(fade) = anim {
			fade.target_alpha = target_alpha;
			fade.fade_spd = fade_spd;
			return;
		}
	}
	obj.anim.anims.push(render::AnimState::FadeTo(render::FadeTo { target_alpha, fade_spd }));
}
