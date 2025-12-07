use super::*;

#[derive(Default)]
pub struct EditorEditState {
	pub game: Box<fx::FxState>,
	pub tool: Tool,

	pub screen_size: Vec2<i32>,
	pub cursor_pos: Vec2<i32>,
	pub mouse_pos: Vec3<f32>,

	pub selected_terrain: Terrain,
	pub selected_ent: EntityHandle,
	pub selected_args: Option<EntityArgs>,
	pub conn_src: Vec2<i32>,

	pub tool_pos: Option<Vec2<i32>>,
	pub input: Input,
}

impl EditorEditState {
	pub fn load_level(&mut self, json: &str) {
		let level_dto: LevelDto = serde_json::from_str(json).unwrap();
		self.game = fx::FxState::new(0, &level_dto, chipcore::RngSeed::System, &tiles::TILES_EDIT);
		self.game.hud_enabled = false;
		self.game.camera.offset = Vec3f(0.0, 0.0 * 32.0, 400.0);
		self.game.camera.set_perspective(false);
		self.game.pause(); // Unlock the camera
	}
	pub fn reload_level(&mut self, json: &str) {
		let level_dto: LevelDto = serde_json::from_str(json).unwrap();
		let old_cam = self.game.camera.clone();
		self.game = fx::FxState::new(0, &level_dto, chipcore::RngSeed::System, &tiles::TILES_EDIT);
		self.game.pause(); // Unlock the camera
		self.game.camera = old_cam;
	}
	pub fn save_level(&self) -> String {
		let mut legend_map = HashMap::new();
		let mut legend = Vec::new();
		legend_map.insert(Terrain::Blank, 0); legend.push(Terrain::Blank);
		legend_map.insert(Terrain::Floor, 1); legend.push(Terrain::Floor);
		let mut idx = 2;
		for &terrain in self.game.gs.field.terrain.iter() {
			if !legend_map.contains_key(&terrain) {
				legend_map.insert(terrain, idx);
				legend.push(terrain);
				idx += 1;
			}
		}
		let data = self.game.gs.field.terrain.iter().map(|&terrain| legend_map[&terrain]).collect();
		let entities = self.game.gs.ents.iter().map(chipcore::Entity::to_entity_args).collect();
		let dto = LevelDto {
			name: self.game.gs.field.name.clone(),
			author: self.game.gs.field.author.clone(),
			hint: self.game.gs.field.hint.clone(),
			password: self.game.gs.field.password.clone(),
			time_limit: self.game.gs.field.time_limit,
			required_chips: self.game.gs.field.required_chips,
			map: FieldDto {
				width: self.game.gs.field.width,
				height: self.game.gs.field.height,
				data,
				legend,
			},
			entities,
			connections: self.game.gs.field.conns.clone(),
			replays: self.game.gs.field.replays.clone(),
			trophies: self.game.gs.field.trophies.clone(),
		};
		serde_json::to_string(&dto).unwrap()
	}
	pub fn set_screen_size(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2::new(width, height);
	}
	pub fn mouse_move(&mut self, mouse_x: i32, mouse_y: i32) {
		let cam = self.game.camera.setup(self.screen_size);
		let ray = cam.viewport_to_ray(Vec2(mouse_x, mouse_y));

		let plane = Plane3::new(Vec3::Z, 0.0);
		if let Some(hit) = ray.trace(&plane) {
			self.mouse_pos = ray.at(hit.distance);
			self.cursor_pos = self.mouse_pos.xy().map(|c| (c / 32.0).floor() as i32);
		}
		else {
			self.mouse_pos = Vec3::ZERO;
			self.cursor_pos = Vec2::ZERO;
		}
	}
	pub fn key_left(&mut self, pressed: bool) {
		self.input.key_left = pressed;
	}
	pub fn key_right(&mut self, pressed: bool) {
		self.input.key_right = pressed;
	}
	pub fn key_up(&mut self, pressed: bool) {
		self.input.key_up = pressed;
	}
	pub fn key_down(&mut self, pressed: bool) {
		self.input.key_down = pressed;
	}
	pub fn key_shift(&mut self, pressed: bool) {
		self.input.key_shift = pressed;
	}
	pub fn think(&mut self) {
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources, time: f64) {
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		if self.input.key_left {
			self.game.camera.target.x -= 5.0;
		}
		if self.input.key_right {
			self.game.camera.target.x += 5.0;
		}
		if self.input.key_up {
			self.game.camera.target.y -= 5.0;
		}
		if self.input.key_down {
			self.game.camera.target.y += 5.0;
		}

		self.game.camera.animate_position(self.game.dt);

		if let Some(tool_pos) = self.tool_pos {
			if tool_pos != self.cursor_pos {
				self.tool_pos = Some(self.cursor_pos);
				match self.tool {
					Tool::Terrain => tool::terrain::think(self),
					Tool::Entity => tool::entity::think(self),
					Tool::Connection => tool::connection::think(self),
				}
			}
		}

		render::drawbg(g, resx);
		self.game.draw(g, resx, time);

		let cam = self.game.camera.setup(self.screen_size);

		let p = self.mouse_pos; {
			let mut cv = shade::im::DrawBuilder::<render::Vertex, render::Uniform>::new();
			cv.viewport = Bounds2::vec(self.screen_size);
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.shader = resx.shader;
			cv.uniform.transform = cam.view_proj;
			cv.uniform.texture = resx.spritesheet_texture;
			cv.uniform.pixel_bias = resx.pixel_art_bias;

			for y in 0..TERRAIN_SAMPLES.len() as i32 {
				for x in 0..2 {
					let terrain = TERRAIN_SAMPLES[y as usize][x as usize];
					let pos = Vec3::new((x - 3) as f32 * 32.0, y as f32 * 32.0, 0.0);
					render::draw_tile(&mut cv, resx, terrain, pos, &self.game.render.tiles);
				}
			}

			for i in 0..ENTITY_SAMPLES.len() as i32 {
				let (_, sprite) = ENTITY_SAMPLES[i as usize];
				let pos = Vec3::new(i as f32 * 32.0, -2.0 * 32.0, 0.0);
				render::draw(&mut cv, resx, pos, sprite, chipty::ModelId::ReallyFlatSprite, 0, 1.0);
			}

			match self.tool {
				Tool::Terrain => {
					render::draw_tile(&mut cv, resx, self.selected_terrain, p, &self.game.render.tiles);
				}
				_ => (),
			}
			g.clear(&shade::ClearArgs {
				surface: shade::Surface::BACK_BUFFER,
				depth: Some(1.0),
				..Default::default()
			});
			cv.draw(g, shade::Surface::BACK_BUFFER);
		}

		{
			let mut cv = shade::im::DrawBuilder::<render::Vertex, render::Uniform>::new();
			cv.viewport = Bounds2::vec(self.screen_size);
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.shader = resx.shader;
			cv.uniform.transform = cam.view_proj;
			cv.uniform.texture = resx.spritesheet_texture;
			cv.uniform.pixel_bias = resx.pixel_art_bias;

			struct ToVertex {
				color: [u8; 4],
			}
			impl shade::d2::ToVertex<render::Vertex> for ToVertex {
				fn to_vertex(&self, pos: Point2<f32>, _: usize) -> render::Vertex {
					render::Vertex { pos: pos.vec3(0.0), uv: Vec2::ZERO, color: self.color }
				}
			}
			let pen = shade::d2::Pen { template: ToVertex { color: [0, 0, 255, 255] } };
			for conn in &self.game.gs.field.conns {
				let src = conn.src.map(|c| c as f32 * 32.0 + 16.0);
				let dest = conn.dest.map(|c| c as f32 * 32.0 + 16.0);
				cv.draw_arrow(&pen, src, dest, 12.0);
			}

			let pen = shade::d2::Pen { template: ToVertex { color: [255, 0, 0, 255] } };
			for ent in self.game.gs.ents.iter() {
				let pos = ent.pos.map(|c| c as f32 * 32.0 + 16.0);
				if let Some(face_dir) = ent.face_dir {
					cv.draw_arrow(&pen, pos, pos + face_dir.to_vec().map(|c| c as f32 * 20.0), 4.0);
				}
				cv.draw_line_rect(&pen, &Bounds2::new(pos - Vec2::dup(4.0), pos + Vec2::dup(4.0)));
			}
			cv.draw(g, shade::Surface::BACK_BUFFER);
		}
	}

	pub fn tool_terrain(&mut self, pressed: bool) {
		if pressed {
			self.tool = Tool::Terrain;
			self.selected_terrain = Terrain::Floor;
			self.tool_pos = None;
		}
	}
	pub fn tool_entity(&mut self, pressed: bool) {
		if pressed {
			self.tool = Tool::Entity;
			self.selected_ent = EntityHandle::INVALID;
			self.selected_args = None;
			self.tool_pos = None;
		}
	}
	pub fn tool_connection(&mut self, pressed: bool) {
		if pressed {
			self.tool = Tool::Connection;
			self.tool_pos = None;
		}
	}

	/// Resizes the playfield.
	///
	/// Positive values = expand, Negative values = crop.
	pub fn resize(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
		let brush = self.game.gs.brush_create();

		let new_width = self.game.gs.field.width + left + right;
		let new_height = self.game.gs.field.height + top + bottom;
		if new_width < chipty::FIELD_MIN_WIDTH || new_width > chipty::FIELD_MAX_WIDTH {
			return;
		}
		if new_height < chipty::FIELD_MIN_HEIGHT || new_height > chipty::FIELD_MAX_HEIGHT {
			return;
		}

		self.game.gs.field.width = new_width;
		self.game.gs.field.height = new_height;
		self.game.gs.field.terrain.clear();
		self.game.gs.field.terrain.resize((new_width * new_height) as usize, self.selected_terrain);
		self.game.gs.field.conns.clear();
		self.game.gs.ents.clear();

		self.game.gs.brush_apply(Vec2i(left, top), &brush);
	}

	pub fn left_click(&mut self, pressed: bool) {
		match self.tool {
			Tool::Terrain => tool::terrain::left_click(self, pressed),
			Tool::Entity => tool::entity::left_click(self, pressed),
			Tool::Connection => tool::connection::left_click(self, pressed),
		}
		self.input.left_click = pressed;
	}
	pub fn right_click(&mut self, pressed: bool) {
		match self.tool {
			Tool::Terrain => tool::terrain::right_click(self, pressed),
			Tool::Entity => tool::entity::right_click(self, pressed),
			Tool::Connection => tool::connection::right_click(self, pressed),
		}
		self.input.right_click = pressed;
	}
	pub fn delete(&mut self, pressed: bool) {
		match self.tool {
			Tool::Terrain => {}
			Tool::Entity => tool::entity::delete(self, pressed),
			Tool::Connection => {}
		}
	}

	pub fn sample(&mut self) {
		let s = self;
		let cursor_pos = s.cursor_pos;

		// Sample from the terrain samples
		if cursor_pos.x < 0 {
			if cursor_pos.x == -3 && cursor_pos.y >= 0 && cursor_pos.y < TERRAIN_SAMPLES.len() as i32 {
				s.tool = Tool::Terrain;
				s.selected_terrain = TERRAIN_SAMPLES[cursor_pos.y as usize][0]
			}
			if cursor_pos.x == -2 && cursor_pos.y >= 0 && cursor_pos.y < TERRAIN_SAMPLES.len() as i32 {
				s.tool = Tool::Terrain;
				s.selected_terrain = TERRAIN_SAMPLES[cursor_pos.y as usize][1];
			}
		}
		// Sample from the entity samples
		else if cursor_pos.y < 0 {
			if cursor_pos.y == -2 && cursor_pos.x >= 0 && cursor_pos.x < ENTITY_SAMPLES.len() as i32 {
				let (kind, _) = ENTITY_SAMPLES[cursor_pos.x as usize];
				s.tool = Tool::Entity;
				s.selected_ent = EntityHandle::INVALID;
				s.selected_args = Some(EntityArgs { kind, pos: cursor_pos, face_dir: None });
			}
		}
		else {
			// Sample from the existing entities
			let ehandle = s.game.gs.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
			if let Some(ehandle) = ehandle {
				if let Some(ent) = s.game.gs.ents.get(ehandle) {
					s.tool = Tool::Entity;
					s.selected_ent = ehandle;
					s.selected_args = Some(ent.to_entity_args());
					println!("Selected: {:?} at {}", ent.kind, ent.pos);
				}
			}
			// Sample from the terrain
			else {
				s.tool = Tool::Terrain;
				s.selected_terrain = s.game.gs.field.get_terrain(cursor_pos);
			}
		}

		println!("Tool: {:?}", s.tool);
	}
}
