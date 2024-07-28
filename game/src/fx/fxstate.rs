use super::*;

#[derive(Default)]
pub struct FxState {
	pub ntime: i32,
	pub time: f32,
	pub dt: f32,
	pub gs: core::GameState,
	pub menu: Option<MainMenu>,
	pub camera: Camera,
	pub objects: ObjectMap,
	pub resources: Resources,
	pub level_index: i32,
	pub next_level_load: f32,
	pub music_enabled: bool,
	pub hud_enabled: bool,
	pub tiles: &'static [TileGfx],
}

impl FxState {
	pub fn init(&mut self) {
		self.tiles = &TILES_PLAY;
		self.level_index = 1;
		self.music_enabled = true;
		// self.menu = Some(MainMenu::default());
	}
	pub fn load_level_by_index(&mut self, level_index: i32) -> bool {
		if let Ok(json) = std::fs::read_to_string(format!("data/cc1/level{}.json", level_index)) {
			self.level_index = level_index;
			self.load_level_from_str(&json);
			return true;
		}
		return false;
	}
	pub fn load_level_from_str(&mut self, json: &str) {
		self.objects.clear();
		self.gs.load(json);
		self.sync(None);
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
	}
	pub fn update(&mut self, input: &core::Input, mut audio: Option<&mut dyn IAudioPlayer>) {
		if let Some(audio) = &mut audio {
			let music = if !self.music_enabled { None } else { Some(match self.level_index.wrapping_sub(1) % 2 {
				0 => MusicId::Chip1,
				_ => MusicId::Chip2,
				// _ => MusicId::Canyon,
			}) };
			audio.play_music(music);
		}

		if let Some(menu) = &mut self.menu {
			menu.update(input, &self.gs.input);
			self.gs.input = input.clone();
		}
		else {
			self.gs.tick(input);
			self.sync(audio);

			if self.gs.ps.activity.is_game_over() && self.time >= self.next_level_load {
				let level_index = self.level_index + if self.gs.ps.activity == core::PlayerActivity::Win { 1 } else { 0 };
				self.load_level_by_index(level_index);
			}
		}
	}
	pub fn sync(&mut self, mut audio: Option<&mut dyn IAudioPlayer>) {
		for ev in &mem::replace(&mut self.gs.events, Vec::new()) {
			println!("GameEvent: {:?}", ev);
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
					match old {
						core::Terrain::BlueFake => blue_wall_cleared(self, pos),
						core::Terrain::HiddenWall => hidden_wall_bumped(self, pos),
						core::Terrain::RecessedWall => recessed_wall_raised(self, pos),
						core::Terrain::ToggleFloor => tw = true,
						core::Terrain::ToggleWall => tw = true,
						core::Terrain::Fire => remove_fire(self, pos),
						_ => {}
					}
					if tw {
						toggle_walls(self);
					}
					match new {
						core::Terrain::Fire => create_fire(self, pos),
						_ => {}
					}
				},
				&core::GameEvent::GameWin { .. } => game_win(self),
				&core::GameEvent::GameOver { .. } => game_win(self),
				&core::GameEvent::SoundFx { sound } => if let Some(ref mut audio) = audio { audio.play(sound) },
				_ => {}
			}
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics) {
		self.ntime += 1;
		let time = self.ntime as f32 / 60.0;
		self.time = time;
		self.dt = 1.0 / 60.0;
		let size = self.resources.screen_size;

		for handle in self.objects.map.keys().cloned().collect::<Vec<_>>() {
			let Some(mut obj) = self.objects.remove(handle) else { continue };
			obj.update(self);
			self.objects.insert(obj);
		}

		g.begin().unwrap();

		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		self.set_game_camera();

		let mut cv = shade::d2::CommandBuffer::<render::Vertex, render::Uniform>::new();
		cv.shader = self.resources.shader;
		cv.depth_test = Some(shade::DepthTest::Less);
		cv.viewport = cvmath::Rect::vec(size);
		// cv.cull_mode = Some(shade::CullMode::CW);
		cv.push_uniform(render::Uniform { transform: self.camera.view_proj_mat, texture: self.resources.tileset, texture_size: self.resources.tileset_size.map(|c| c as f32).into() });
		render::field(&mut cv, self, time);
		cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();

		if self.hud_enabled {
			self.render_ui(g);

			if let Some(menu) = &mut self.menu {
				menu.draw(g, &self.resources);
			}
		}

		g.end().unwrap();

		self.objects.map.retain(|_, obj| obj.live);
	}
}
