use super::*;

#[derive(Default)]
pub struct FxState {
	pub ntime: i32,
	pub time: f32,
	pub dt: f32,
	pub gs: core::GameState,
	pub gs_realtime: f32,
	pub camera: Camera,
	pub objects: ObjectMap,
	pub level_number: i32,
	pub next_level_load: f32,
	pub game_win: bool,
	pub music_enabled: bool,
	pub music: Option<data::MusicId>,
	pub hud_enabled: bool,
	pub darken: bool,
	pub darken_changed: bool,
	pub darken_time: f32,
	pub tiles: &'static [TileGfx],
	pub events: Vec<FxEvent>,
	pub axes: Option<shade::d3::axes::AxesModel>,
}

impl FxState {
	pub fn init(&mut self) {
		self.tiles = &TILES_PLAY;
		self.level_number = 0;
		self.music_enabled = false;
	}
	pub fn parse_level(&mut self, level_number: i32, json: &str) {
		self.objects.clear();
		self.gs.parse(json, core::RngSeed::System);
		self.sync();
		self.camera.eye_offset = Vec3::new(0.0, 2.0 * 32.0, 400.0);

		for y in 0..self.gs.field.height {
			for x in 0..self.gs.field.width {
				let index = (y * self.gs.field.width + x) as usize;
				let terrain = self.gs.field.terrain[index];
				match terrain {
					core::Terrain::Fire => create_fire(self, Vec2::new(x, y)),
					core::Terrain::ToggleFloor => create_toggle_floor(self, Vec2(x, y)),
					core::Terrain::ToggleWall => create_toggle_wall(self, Vec2(x, y)),
					_ => {}
				}
			}
		}

		self.hud_enabled = true;
		self.level_number = level_number;
		self.next_level_load = 0.0;
		self.darken = true;
		self.darken_time = -1.0;
	}
	pub fn pause(&mut self) {
		// if matches!(self.gs.ts, core::TimeState::Running) {
			self.gs.ts = core::TimeState::Paused;
			self.events.push(FxEvent::Pause);
			self.hud_enabled = false;
		// }
	}
	pub fn unpause(&mut self) {
		if matches!(self.gs.ts, core::TimeState::Paused) {
			self.gs.ts = core::TimeState::Running;
			self.events.push(FxEvent::Unpause);
			self.hud_enabled = true;
		}
	}
	pub fn think(&mut self, input: &Input) {
		let music = if self.music_enabled {
			let music = match self.level_number.wrapping_sub(1) % 2 {
				0 => data::MusicId::Chip1,
				_ => data::MusicId::Chip2,
				// _ => data::MusicId::Canyon,
			};
			Some(music)
		}
		else {
			None
		};
		if music != self.music {
			self.music = music;
			self.events.push(FxEvent::PlayMusic { music });
		}

		if input.start.is_pressed() && !self.gs.ps.activity.is_game_over() {
			self.pause();
		}

		self.gs.tick(&core::Input {
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
				&core::GameEvent::EntityCreated { entity, kind } => entity_created(self, entity, kind),
				&core::GameEvent::EntityRemoved { entity, kind } => entity_removed(self, entity, kind),
				&core::GameEvent::EntityStep { entity } => entity_step(self, entity),
				&core::GameEvent::EntityTurn { entity } => entity_face_dir(self, entity),
				&core::GameEvent::EntityHidden { entity, hidden } => entity_hidden(self, entity, hidden),
				&core::GameEvent::EntityTeleport { entity } => entity_teleport(self, entity),
				&core::GameEvent::EntityDrown { entity } => entity_drown(self, entity),
				&core::GameEvent::PlayerActivity { player } => player_activity(self, player),
				&core::GameEvent::ItemPickup { entity, item } => item_pickup(self, entity, item),
				&core::GameEvent::LockOpened { pos, key } => lock_opened(self, pos, key),
				&core::GameEvent::TerrainUpdated { pos, old, new } => {
					let mut tw = false;
					match (old, new) {
						(core::Terrain::FakeBlueWall, _) => blue_wall_cleared(self, pos),
						(core::Terrain::HiddenWall, _) => hidden_wall_bumped(self, pos),
						(core::Terrain::ToggleFloor, _) => tw = true,
						(core::Terrain::ToggleWall, _) => tw = true,
						(core::Terrain::Fire, _) => remove_fire(self, pos),
						(_, core::Terrain::Fire) => create_fire(self, pos),
						(core::Terrain::RecessedWall, core::Terrain::Wall) => recessed_wall_raised(self, pos),
						_ => {}
					}
					if tw {
						toggle_walls(self);
					}
				},
				&core::GameEvent::GameWin { .. } => game_win(self),
				&core::GameEvent::GameOver { .. } => game_over(self),
				&core::GameEvent::SoundFx { sound } => self.events.push(FxEvent::PlaySound { sound }),
				_ => {}
			}
		}

		if self.next_level_load != 0.0 && self.time > self.next_level_load {
			self.next_level_load = 0.0;
			self.events.push(if self.game_win { FxEvent::GameWin } else { FxEvent::GameOver });
		}
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

		self.set_game_camera();

		let viewport = Bounds2::vec(resx.screen_size);
		let cam = self.camera.setup(viewport.size());

		let mut cv = shade::d2::CommandBuffer::<render::Vertex, render::Uniform>::new();
		cv.shader = resx.shader;
		cv.depth_test = Some(shade::DepthTest::Less);
		cv.viewport = viewport;
		cv.cull_mode = Some(shade::CullMode::CW);
		cv.push_uniform(render::Uniform { transform: cam.view_proj, texture: resx.tileset });
		render::field(&mut cv, self, time);
		cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();

		if self.axes.is_none() {
			let shader = g.shader_create(None, shade::gl::shaders::AXES_VS, shade::gl::shaders::AXES_FS).unwrap();
			self.axes = Some(shade::d3::axes::AxesModel::create(g, shader));
		}
		if let Some(axes) = &self.axes {
			axes.draw(g, &cam, &shade::d3::axes::AxesInstance {
				local: Transform3f::scale(Vec3::dup(50.0)),
				depth_test: None,
			});
		}

		if self.hud_enabled {
			self.render_ui(g, resx);
		}

		self.objects.map.retain(|_, obj| obj.live);
	}
}
