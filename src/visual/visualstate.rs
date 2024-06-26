use super::*;

#[derive(Default)]
pub struct VisualState {
	pub time: f32,
	pub dt: f32,
	pub game: core::GameState,
	pub camera: Camera,
	pub objects: ObjectMap,
	pub resources: Resources,
	pub tiles: &'static [TileGfx],
}

impl VisualState {
	pub fn init(&mut self) {
		self.tiles = &TILES_PLAY;
	}
	pub fn load_level(&mut self, json: &str) {
		self.game.load(json);
		self.sync(&self.game.events.clone());
		self.camera.eye_offset = Vec3::new(0.0, 2.0 * 32.0, 400.0);

		for y in 0..self.game.field.height {
			for x in 0..self.game.field.width {
				let index = (y * self.game.field.width + x) as usize;
				let terrain = self.game.field.terrain[index];
				match terrain {
					core::Terrain::Fire => create_fire(self, Vec2::new(x, y)),
					core::Terrain::ToggleFloor => create_toggle_floor(self, Vec2(x, y)),
					core::Terrain::ToggleWall => create_toggle_wall(self, Vec2(x, y)),
					_ => {}
				}
			}
		}
	}
	pub fn update(&mut self, input: &core::Input) {
		self.game.tick(input);
		self.sync(&self.game.events.clone());
	}
	pub fn sync(&mut self, events: &Vec<core::GameEvent>) {
		for ev in events {
			println!("Event: {:?}", ev);
			match ev {
				&core::GameEvent::EntityCreated { entity } => entity_created(self, entity),
				&core::GameEvent::EntityRemoved { entity } => entity_removed(self, entity),
				&core::GameEvent::EntityStep { entity } => entity_step(self, entity),
				&core::GameEvent::EntityFaceDir { entity } => entity_face_dir(self, entity),
				&core::GameEvent::EntityHidden { entity, hidden } => entity_hidden(self, entity, hidden),
				&core::GameEvent::EntityTeleport { entity } => entity_teleport(self, entity),
				&core::GameEvent::PlayerAction { player } => entity_face_dir(self, player),
				&core::GameEvent::ItemPickup { player, .. } => item_pickup(self, player),
				&core::GameEvent::LockRemoved { pos, key } => lock_removed(self, pos, key),
				&core::GameEvent::BlueWallCleared { pos } => blue_wall_cleared(self, pos),
				&core::GameEvent::HiddenWallBumped { pos } => hidden_wall_bumped(self, pos),
				&core::GameEvent::RecessedWallRaised { pos } => recessed_wall_raised(self, pos),
				&core::GameEvent::GreenButton { .. } => toggle_walls(self),
				_ => {}
			}
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics) {
		let time = self.game.time as f32 / 60.0;
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

		let mut cv = shade::d2::Canvas::<render::Vertex, render::Uniform>::new();
		cv.shader = self.resources.shader;
		cv.depth_test = Some(shade::DepthTest::Less);
		cv.viewport = cvmath::Rect::vec(cvmath::Vec2(size.x as i32, size.y as i32));
		// cv.cull_mode = Some(shade::CullMode::CW);
		cv.push_uniform(render::Uniform { transform: self.camera.view_proj_mat, texture: self.resources.tileset, texture_size: self.resources.tileset_size.map(|c| c as f32).into() });
		render::field(&mut cv, self, time);
		cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();

		g.end().unwrap();

		self.objects.map.retain(|_, obj| obj.live);
	}
}
