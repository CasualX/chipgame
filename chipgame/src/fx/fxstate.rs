use super::*;

pub enum EffectType {
	Splash,
	Sparkles,
	Fireworks,
}

pub struct Effect {
	pub ty: EffectType,
	pub pos: Vec3f,
	pub start: f32,
}

#[derive(Default)]
pub struct FxState {
	pub ntime: i32,
	pub time: f32,
	pub dt: f32,
	pub gs: chipcore::GameState,
	pub gs_realtime: f32,
	pub camera: PlayCamera,
	pub objects: ObjectMap,
	pub effects: Vec<Effect>,
	pub level_number: i32,
	pub next_level_load: f32,
	pub game_win: bool,
	pub music_enabled: bool,
	pub music: Option<chipty::MusicId>,
	pub hud_enabled: bool,
	pub darken: bool,
	pub darken_changed: bool,
	pub darken_time: f32,
	pub tiles: &'static [TileGfx],
	pub events: Vec<FxEvent>,
}

impl FxState {
	pub fn init(&mut self) {
		self.tiles = &TILES_PLAY;
		self.level_number = 0;
		self.music_enabled = false;
	}
	pub fn parse_level(&mut self, level_number: i32, json: &str) {
		self.objects.clear();
		self.gs.parse(json, chipcore::RngSeed::System);

		for y in 0..self.gs.field.height {
			for x in 0..self.gs.field.width {
				let index = (y * self.gs.field.width + x) as usize;
				let terrain = self.gs.field.terrain[index];
				match terrain {
					chipty::Terrain::Fire => create_fire(self, Vec2::new(x, y)),
					chipty::Terrain::ToggleFloor => create_toggle_floor(self, Vec2(x, y)),
					chipty::Terrain::ToggleWall => create_toggle_wall(self, Vec2(x, y)),
					_ => {}
				}
			}
		}

		self.camera = PlayCamera::default(); // Reset the camera, adjusted when a player entity is created
		self.sync();

		self.hud_enabled = true;
		self.level_number = level_number;
		self.next_level_load = 0.0;
		self.darken = true;
		self.darken_time = -1.0;
	}
	pub fn scout(&mut self) {
		self.gs.ts = chipcore::TimeState::Paused;
		self.events.push(FxEvent::Scout);
		self.hud_enabled = true;
	}
	pub fn pause(&mut self) {
		self.gs.ts = chipcore::TimeState::Paused;
		self.events.push(FxEvent::Pause);
		self.hud_enabled = false;
	}
	pub fn unpause(&mut self) {
		if matches!(self.gs.ts, chipcore::TimeState::Paused) {
			self.gs.ts = chipcore::TimeState::Running;
			self.events.push(FxEvent::Unpause);
			self.hud_enabled = true;

			// Center camera on player again
			let Some(&obj_handle) = self.objects.lookup.get(&self.gs.ps.ehandle) else { return };
			let Some(obj) = self.objects.get_mut(obj_handle) else { return };
			self.camera.teleport(obj.lerp_pos + Vec3(16.0, 16.0, 0.0));
		}
	}
	pub fn think(&mut self, input: &Input) {
		if !self.gs.ps.activity.is_game_over() {
			if input.start.is_pressed() {
				self.pause();
			}
			else if input.select.is_pressed() {
				self.scout();
			}
		}

		self.gs.tick(&chipcore::Input {
			a: input.a.is_held(),
			b: input.b.is_held(),
			up: input.up.is_held(),
			down: input.down.is_held(),
			left: input.left.is_held(),
			right: input.right.is_held(),
			start: input.start.is_held(),
			select: input.select.is_held(),
		});
		self.sync();
	}
	pub fn sync(&mut self) {
		for ev in &self.gs.events.take() {
			eprintln!("GameEvent: {:?}", ev);
			match ev {
				&chipcore::GameEvent::EntityCreated { entity, kind } => entity_created(self, entity, kind),
				&chipcore::GameEvent::EntityRemoved { entity, kind } => entity_removed(self, entity, kind),
				&chipcore::GameEvent::EntityStep { entity } => entity_step(self, entity),
				&chipcore::GameEvent::EntityTurn { entity } => entity_face_dir(self, entity),
				&chipcore::GameEvent::EntityHidden { entity, hidden } => entity_hidden(self, entity, hidden),
				&chipcore::GameEvent::EntityTeleport { entity } => entity_teleport(self, entity),
				&chipcore::GameEvent::EntityDrown { entity } => entity_drown(self, entity),
				&chipcore::GameEvent::PlayerActivity { player } => player_activity(self, player),
				&chipcore::GameEvent::ItemPickup { entity, item } => item_pickup(self, entity, item),
				&chipcore::GameEvent::LockOpened { pos, key } => lock_opened(self, pos, key),
				&chipcore::GameEvent::FireHidden { pos, hidden } => fire_hidden(self, pos, hidden),
				&chipcore::GameEvent::TerrainUpdated { pos, old, new } => {
					let mut tw = false;
					match (old, new) {
						(chipty::Terrain::FakeBlueWall, _) => blue_wall_cleared(self, pos),
						(chipty::Terrain::HiddenWall, _) => hidden_wall_bumped(self, pos),
						(chipty::Terrain::ToggleFloor, _) => tw = true,
						(chipty::Terrain::ToggleWall, _) => tw = true,
						(chipty::Terrain::Fire, _) => remove_fire(self, pos),
						(_, chipty::Terrain::Fire) => create_fire(self, pos),
						(chipty::Terrain::RecessedWall, chipty::Terrain::Wall) => recessed_wall_raised(self, pos),
						_ => {}
					}
					if tw {
						toggle_walls(self);
					}
				},
				&chipcore::GameEvent::GameWin { .. } => game_win(self),
				&chipcore::GameEvent::GameOver { .. } => game_over(self),
				&chipcore::GameEvent::SoundFx { sound } => self.events.push(FxEvent::PlaySound { sound }),
				&chipcore::GameEvent::BombExplode { pos } => effect(self, pos, EffectType::Sparkles),
				&chipcore::GameEvent::WaterSplash { pos } => effect(self, pos, EffectType::Splash),
				&chipcore::GameEvent::Fireworks { pos } => effect(self, pos, EffectType::Fireworks),
				_ => {}
			}
		}

		if self.next_level_load != 0.0 && self.time > self.next_level_load {
			self.next_level_load = 0.0;
			self.events.push(if self.game_win { FxEvent::GameWin } else { FxEvent::GameOver });
		}
	}
	pub fn follow_player(&mut self) {
		if matches!(self.gs.ts, chipcore::TimeState::Paused) {
			return;
		}
		if let Some(obj) = self.objects.lookup.get(&self.gs.ps.ehandle).and_then(|h| self.objects.get(*h)) {
			self.camera.set_target(obj.lerp_pos + Vec3(16.0, 16.0, 0.0));
		}
	}
	pub fn scout_dir(&mut self, dir: chipty::Compass) {
		self.camera.set_target(self.camera.target + dir.to_vec().vec3(0).cast::<f32>() * 2.0);
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		self.ntime += 1;
		let time = self.ntime as f32 / 60.0;
		self.time = time;
		self.dt = 1.0 / 60.0;

		for handle in self.objects.map.keys().cloned().collect::<Vec<_>>() {
			let Some(mut obj) = self.objects.remove(handle) else { continue };
			obj.update(self);
			self.objects.insert(obj);
		}

		if self.gs.time != 0 {
			self.camera.animate_blend();
		}
		self.camera.animate_position();

		let camera = self.camera.setup(resx.viewport.size());

		{
			let mut cv = shade::d2::DrawBuilder::<render::Vertex, render::Uniform>::new();
			cv.viewport = resx.viewport;
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.cull_mode = Some(shade::CullMode::CW);
			cv.shader = resx.shader;
			cv.uniform.transform = camera.view_proj;
			cv.uniform.texture = resx.tileset;
			cv.uniform.pixel_bias = resx.pixel_art_bias;
			cv.uniform.pixel_bias = resx.pixel_art_bias;
			render::field(&mut cv, self, time, 1.0);
			cv.draw(g, shade::Surface::BACK_BUFFER);
		}

		// Render the effects
		{
			let mut cv = shade::d2::DrawBuilder::<Vertex, Uniform>::new();
			cv.viewport = resx.viewport;
			cv.blend_mode = shade::BlendMode::Solid;
			cv.depth_test = Some(shade::DepthTest::Always);
			// cv.cull_mode = Some(shade::CullMode::CW);

			cv.shader = resx.shader;
			cv.uniform.transform = camera.view_proj;
			cv.uniform.texture = resx.effects;

			for efx in &self.effects {
				render::draw_effect(&mut cv, efx, time);
			}
			cv.draw(g, shade::Surface::BACK_BUFFER);
		}
		self.effects.retain(|efx| time < efx.start + 1.0);

		if self.hud_enabled {
			self.render_ui(g, resx);
		}

		self.objects.map.retain(|_, obj| obj.live);
	}
}
