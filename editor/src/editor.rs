use std::collections::HashMap;
use chipgame::fx::*;
use chipgame::core;
use cvmath::*;

#[derive(Clone, Debug)]
pub enum Tool {
	Terrain(core::Terrain),
	Entity(core::EntityArgs),
}
impl Default for Tool {
	fn default() -> Self {
		Tool::Terrain(core::Terrain::Floor)
	}
}

static TERRAIN_SAMPLES: [[core::Terrain; 2]; 21] = [
	[core::Terrain::Blank, core::Terrain::Floor],
	[core::Terrain::Dirt, core::Terrain::Gravel],
	[core::Terrain::Wall, core::Terrain::CloneMachine],
	[core::Terrain::HiddenWall, core::Terrain::InvisWall],
	[core::Terrain::BlueWall, core::Terrain::BlueFake],
	[core::Terrain::BlueLock, core::Terrain::RedLock],
	[core::Terrain::GreenLock, core::Terrain::YellowLock],
	[core::Terrain::Exit, core::Terrain::Hint],
	[core::Terrain::Water, core::Terrain::Fire],
	[core::Terrain::PanelE, core::Terrain::PanelS],
	[core::Terrain::PanelN, core::Terrain::PanelW],
	[core::Terrain::PanelSE, core::Terrain::Ice],
	[core::Terrain::IceNW, core::Terrain::IceNE],
	[core::Terrain::IceSW, core::Terrain::IceSE],
	[core::Terrain::ToggleFloor, core::Terrain::ToggleWall],
	[core::Terrain::GreenButton, core::Terrain::RedButton],
	[core::Terrain::BrownButton, core::Terrain::BlueButton],
	[core::Terrain::BearTrap, core::Terrain::RecessedWall],
	[core::Terrain::Teleport, core::Terrain::ForceRandom],
	[core::Terrain::ForceE, core::Terrain::ForceS],
	[core::Terrain::ForceN, core::Terrain::ForceW],
];

static ENTITY_SAMPLES: [(core::EntityKind, chipgame::fx::Sprite); 23] = [
	(core::EntityKind::Player, chipgame::fx::Sprite::PlayerWalkNeutral),
	(core::EntityKind::Chip, chipgame::fx::Sprite::Chip),
	(core::EntityKind::Socket, chipgame::fx::Sprite::Socket),
	(core::EntityKind::Block, chipgame::fx::Sprite::Block),
	(core::EntityKind::Flippers, chipgame::fx::Sprite::PowerFlippers),
	(core::EntityKind::FireBoots, chipgame::fx::Sprite::PowerFireBoots),
	(core::EntityKind::IceSkates, chipgame::fx::Sprite::PowerIceSkates),
	(core::EntityKind::SuctionBoots, chipgame::fx::Sprite::PowerSuctionBoots),
	(core::EntityKind::BlueKey, chipgame::fx::Sprite::BlueKey),
	(core::EntityKind::RedKey, chipgame::fx::Sprite::RedKey),
	(core::EntityKind::GreenKey, chipgame::fx::Sprite::GreenKey),
	(core::EntityKind::YellowKey, chipgame::fx::Sprite::YellowKey),
	(core::EntityKind::Thief, chipgame::fx::Sprite::Thief),
	(core::EntityKind::Bomb, chipgame::fx::Sprite::Bomb),
	(core::EntityKind::Bug, chipgame::fx::Sprite::BugUp),
	(core::EntityKind::FireBall, chipgame::fx::Sprite::FireBall),
	(core::EntityKind::PinkBall, chipgame::fx::Sprite::PinkBall),
	(core::EntityKind::Tank, chipgame::fx::Sprite::TankUp),
	(core::EntityKind::Glider, chipgame::fx::Sprite::GliderUp),
	(core::EntityKind::Teeth, chipgame::fx::Sprite::TeethUp),
	(core::EntityKind::Walker, chipgame::fx::Sprite::WalkerUpDown),
	(core::EntityKind::Blob, chipgame::fx::Sprite::Blob),
	(core::EntityKind::Paramecium, chipgame::fx::Sprite::ParameciumUpDown),
];

#[derive(Default)]
struct Input {
	left_click: bool,
	right_click: bool,
	key_left: bool,
	key_right: bool,
	key_up: bool,
	key_down: bool,
}

#[derive(Default)]
pub struct EditorState {
	game: VisualState,
	tool: Tool,

	screen_size: Vec2<i32>,
	cursor_pos: Vec2<i32>,
	mouse_pos: Vec3<f32>,
	selected_ent: core::EntityHandle,

	tool_pos: Option<Vec2<i32>>,
	conn_src: Vec2<i32>,
	input: Input,
}

impl EditorState {
	pub fn init(&mut self, resources: Resources) {
		self.game.resources = resources;
		self.game.tiles = &TILES_EDIT;
	}
	pub fn load_level(&mut self, json: &str) {
		self.game.load_level(json);
	}
	pub fn save_level(&self) -> String {
		let mut legend_map = HashMap::new();
		let mut legend = Vec::new();
		legend_map.insert(core::Terrain::Blank, 0); legend.push(core::Terrain::Blank);
		legend_map.insert(core::Terrain::Floor, 1); legend.push(core::Terrain::Floor);
		let mut idx = 2;
		for &terrain in self.game.gs.field.terrain.iter() {
			if !legend_map.contains_key(&terrain) {
				legend_map.insert(terrain, idx);
				legend.push(terrain);
				idx += 1;
			}
		}
		let data = self.game.gs.field.terrain.iter().map(|&terrain| legend_map[&terrain]).collect();

		let mut entities: Vec<_> = self.game.gs.ents.iter().map(|ent| core::EntityArgs {
			kind: ent.kind,
			pos: ent.pos,
			face_dir: ent.face_dir,
		}).collect();
		entities.sort_unstable_by_key(|ent| (ent.kind as i32, ent.pos.y, ent.pos.x));

		let dto = core::FieldDto {
			name: self.game.gs.field.name.clone(),
			hint: self.game.gs.field.hint.clone(),
			password: self.game.gs.field.password.clone(),
			seed: urandom::new().next_u64(),
			time: self.game.gs.field.time,
			chips: self.game.gs.field.chips,
			map: core::MapDto {
				width: self.game.gs.field.width,
				height: self.game.gs.field.height,
				data,
				legend,
			},
			entities,
			connections: self.game.gs.field.conns.clone(),
		};
		serde_json::to_string(&dto).unwrap()
	}
	pub fn set_screen_size(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2::new(width, height);
	}
	pub fn mouse_move(&mut self, mouse_x: i32, mouse_y: i32) {
		let ndc_x = (mouse_x as f32 / self.screen_size.x as f32 - 0.5) * 2.0;
		let ndc_y = (mouse_y as f32 / self.screen_size.y as f32 - 0.5) * -2.0;

		let x = ndc_x / self.game.camera.proj_mat.a11;
		let y = ndc_y / self.game.camera.proj_mat.a22;
		let dir = (self.game.camera.view_mat.inverse() * Vec4::new(x, y, -1.0, 1.0)).xyz().normalize();

		let ray = Ray::new(self.game.camera.target + self.game.camera.eye_offset, dir);
		let plane = Plane::new(Vec3::Z, 0.0);
		let mut hits = [TraceHit::default(); 2];
		if ray.trace(&plane, &mut hits) > 0 {
			self.mouse_pos = ray.at(hits[0].distance);
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
	pub fn render(&mut self, g: &mut shade::Graphics) {

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

		if let Some(tool_pos) = self.tool_pos {
			if tool_pos != self.cursor_pos {
				self.use_tool();
			}
		}

		self.game.camera.eye_offset = Vec3::<f32>(0.0, 8.0 * 32.0, 400.0) * 2.0;
		self.game.camera.object_h = None;

		self.game.draw(g);

		g.begin().unwrap();

		let p = self.mouse_pos; {
			let mut cv = shade::d2::Canvas::<render::Vertex, render::Uniform>::new();
			cv.shader = self.game.resources.shader;
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.viewport = cvmath::Rect::vec(cvmath::Vec2(self.screen_size.x as i32, self.screen_size.y as i32));
			cv.push_uniform(render::Uniform { transform: self.game.camera.view_proj_mat, texture: self.game.resources.tileset, texture_size: self.game.resources.tileset_size.map(|c| c as f32).into() });
			{
				let mut x = cv.begin(shade::PrimType::Triangles, 4, 2);
				x.add_indices_quad();
				let s = 2.0;
				let z = 40.0;
				x.add_vertex(render::Vertex { pos: Vec3::new(p.x-s, p.y-s, p.z), uv: Vec2::new(0.0, 0.0), color: [255, 0, 0, 255] });
				x.add_vertex(render::Vertex { pos: Vec3::new(p.x+s, p.y-s, p.z), uv: Vec2::new(1.0, 0.0), color: [255, 0, 0, 255] });
				x.add_vertex(render::Vertex { pos: Vec3::new(p.x+s, p.y+s, p.z + z), uv: Vec2::new(1.0, 1.0), color: [255, 0, 0, 255] });
				x.add_vertex(render::Vertex { pos: Vec3::new(p.x-s, p.y+s, p.z + z), uv: Vec2::new(0.0, 1.0), color: [255, 0, 0, 255] });
			}

			for y in 0..TERRAIN_SAMPLES.len() as i32 {
				for x in 0..2 {
					let terrain = TERRAIN_SAMPLES[y as usize][x as usize];
					let pos = Vec3::new((x - 3) as f32 * 32.0, y as f32 * 32.0, 0.0);
					render::draw_tile(&mut cv, terrain, pos, &self.game.tiles);
				}
			}

			for i in 0..ENTITY_SAMPLES.len() as i32 {
				let (_, sprite) = ENTITY_SAMPLES[i as usize];
				let pos = Vec3::new(i as f32 * 32.0, -2.0 * 32.0, 0.0);
				render::draw(&mut cv, pos, sprite, chipgame::fx::Model::ReallyFlatSprite, 1.0);
			}

			match self.tool {
				Tool::Terrain(index) => {
					render::draw_tile(&mut cv, index, p, &self.game.tiles);
				}
				_ => (),
			}
			g.clear(&shade::ClearArgs {
				surface: shade::Surface::BACK_BUFFER,
				depth: Some(1.0),
				..Default::default()
			}).unwrap();
			cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
		}

		{
			let mut cv = shade::d2::Canvas::<render::Vertex, render::Uniform>::new();
			cv.shader = self.game.resources.shader;
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.viewport = cvmath::Rect::vec(cvmath::Vec2(self.screen_size.x as i32, self.screen_size.y as i32));
			cv.push_uniform(render::Uniform { transform: self.game.camera.view_proj_mat, texture: self.game.resources.tileset, texture_size: self.game.resources.tileset_size.map(|c| c as f32).into() });

			struct ToVertex {
				color: [u8; 4],
			}
			impl shade::d2::ToVertex<render::Vertex> for ToVertex {
				fn to_vertex(&self, pos: Point2<f32>, _: usize) -> render::Vertex {
					render::Vertex { pos: pos.vec3(0.0), uv: Vec2::ZERO, color: self.color }
				}
			}
			let pen = shade::d2::Pen { template: ToVertex { color: [0, 0, 255, 255] }, segments: 0 };
			for conn in &self.game.gs.field.conns {
				let src = conn.src.map(|c| c as f32 * 32.0 + 16.0);
				let dest = conn.dest.map(|c| c as f32 * 32.0 + 16.0);
				cv.draw_arrow(&pen, src, dest, 12.0);
			}

			let pen = shade::d2::Pen { template: ToVertex { color: [255, 0, 0, 255] }, segments: 0 };
			for ent in self.game.gs.ents.iter() {
				if let Some(face_dir) = ent.face_dir {
					let pos = ent.pos.map(|c| c as f32 * 32.0 + 16.0);
					cv.draw_arrow(&pen, pos, pos + face_dir.to_vec().map(|c| c as f32 * 20.0), 4.0);
				}
			}
			cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
		}

		g.end().unwrap();
	}

	pub fn left_click(&mut self, pressed: bool) {
		let cursor = self.cursor_pos;
		self.tool_pos = None;

		if pressed {
			if cursor.x < 0 || cursor.y < 0 {
				self.sample();
			}
			else {
				self.use_tool_start();
			}
		}
		else if self.tool_pos.is_some() {
			self.use_tool_end();
		}

		self.input.left_click = pressed;
	}
	pub fn right_click(&mut self, pressed: bool) {
		let s = self;
		if pressed {
			let selected_ent = s.selected_ent;
			s.sample();

			if selected_ent == s.selected_ent {
				if let Some(ent_args) = s.game.gs.entity_remove(s.selected_ent) {
					let new_args = core::EntityArgs { kind: ent_args.kind, pos: s.cursor_pos, face_dir: next_face_dir(ent_args.face_dir) };
					let ehandle = s.game.gs.entity_create(&new_args);
					s.game.sync(None);
					s.selected_ent = ehandle;
					if let Some(new_args) = s.game.gs.ents.get(ehandle).map(|ent| ent.to_entity_args()) {
						s.tool = Tool::Entity(new_args);
						println!("FaceDir: {:?} to {:?}", ent_args.kind, new_args.face_dir);
					}
				}
			}
		}
		else if let Some(ent) = s.game.gs.ents.get(s.selected_ent) {
			if ent.pos != s.cursor_pos {
				if let Some(ent_args) = s.game.gs.entity_remove(s.selected_ent) {
					let new_args = core::EntityArgs { kind: ent_args.kind, pos: s.cursor_pos, face_dir: ent_args.face_dir };
					s.game.gs.entity_create(&new_args);
					s.game.sync(None);
					println!("Moved: {:?} to {}", ent_args.kind, s.cursor_pos);
				}
			}
		}

		s.input.right_click = pressed;
	}
	pub fn delete(&mut self, pressed: bool) {
		if pressed {
			if let Some(ent) = self.game.gs.ents.get(self.selected_ent) {
				let kind = ent.kind;
				let pos = ent.pos;
				self.game.gs.entity_remove(self.selected_ent);
				self.game.sync(None);
				println!("Deleted: {:?} at {}", kind, pos);
			}
			self.selected_ent = core::EntityHandle::INVALID;
		}
	}

	pub fn middle_click(&mut self, pressed: bool) {
		if pressed {
			self.conn_src = self.cursor_pos;
		}
		else {
			let new_conn = core::Conn { src: self.conn_src, dest: self.cursor_pos };

			if new_conn.src != new_conn.dest {
				if let Some(index) = self.game.gs.field.conns.iter().position(|conn| conn == &new_conn) {
					self.game.gs.field.conns.remove(index);
				}
				else {
					self.game.gs.field.conns.push(new_conn);
				}
			}
		}
	}

	/// Right click to sample the terrain or entity at the cursor as the current tool
	fn sample(&mut self) {
		let s = self;
		let cursor = s.cursor_pos;

		// Sample from the terrain samples
		if cursor.x < 0 {
			if cursor.x == -3 && cursor.y >= 0 && cursor.y < TERRAIN_SAMPLES.len() as i32 {
				s.tool = Tool::Terrain(TERRAIN_SAMPLES[cursor.y as usize][0]);
			}
			if cursor.x == -2 && cursor.y >= 0 && cursor.y < TERRAIN_SAMPLES.len() as i32 {
				s.tool = Tool::Terrain(TERRAIN_SAMPLES[cursor.y as usize][1]);
			}
		}
		// Sample from the entity samples
		else if cursor.y < 0 {
			if cursor.y == -2 && cursor.x >= 0 && cursor.x < ENTITY_SAMPLES.len() as i32 {
				let (kind, _) = ENTITY_SAMPLES[cursor.x as usize];
				s.tool = Tool::Entity(core::EntityArgs { kind, pos: Vec2::ZERO, face_dir: None });
			}
		}
		else {
			// Sample from the existing entities
			let ehandle = s.game.gs.ents.iter().find_map(|ent| if ent.pos == cursor { Some(ent.handle) } else { None });
			if let Some(ehandle) = ehandle {
				if let Some(ent) = s.game.gs.ents.get(ehandle) {
					s.selected_ent = ehandle;
					println!("Selected: {:?} at {}", ent.kind, ent.pos);
					s.tool = Tool::Entity(ent.to_entity_args());
				}
			}
			// Sample from the terrain
			else {
				let terrain = s.game.gs.field.get_terrain(cursor);
				s.tool = Tool::Terrain(terrain);
			}
		}

		println!("Tool: {:?}", s.tool);
	}

	fn use_tool_start(&mut self) {
		self.use_tool();
	}
	fn use_tool(&mut self) {
		let cursor_pos = self.cursor_pos;
		match self.tool {
			Tool::Terrain(terrain) => {
				self.game.gs.field.set_terrain(cursor_pos, terrain);
			}
			Tool::Entity(ent_args) => {
				// Remove any existing entities at this position
				let mut kind = None;
				let handles = self.game.gs.ents.iter().filter_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None }).collect::<Vec<_>>();
				for ehandle in handles {
					kind = self.game.gs.ents.get(ehandle).map(|ent| ent.kind);
					self.game.gs.entity_remove(ehandle);
				}
				if kind != Some(ent_args.kind) {
					// Create the new entity
					self.game.gs.entity_create(&core::EntityArgs { kind: ent_args.kind, pos: cursor_pos, face_dir: ent_args.face_dir });
				}
				self.game.sync(None);
			}
		}
		self.tool_pos = Some(cursor_pos);
	}
	fn use_tool_end(&mut self) {

	}
}

fn next_face_dir(face_dir: Option<core::Compass>) -> Option<core::Compass> {
	match face_dir {
		Some(core::Compass::Up) => Some(core::Compass::Right),
		Some(core::Compass::Right) => Some(core::Compass::Down),
		Some(core::Compass::Down) => Some(core::Compass::Left),
		Some(core::Compass::Left) => None,
		None => Some(core::Compass::Up),
	}
}
