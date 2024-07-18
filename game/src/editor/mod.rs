use std::collections::HashMap;
use crate::fx;
use crate::core;
use cvmath::*;

mod tool;
mod tiles;

#[derive(Clone, Debug)]
pub enum Tool {
	Terrain,
	Entity,
	Connection,
}
impl Default for Tool {
	fn default() -> Self {
		Tool::Terrain
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

static ENTITY_SAMPLES: [(core::EntityKind, fx::Sprite); 23] = [
	(core::EntityKind::Player, fx::Sprite::PlayerWalkNeutral),
	(core::EntityKind::Chip, fx::Sprite::Chip),
	(core::EntityKind::Socket, fx::Sprite::Socket),
	(core::EntityKind::Block, fx::Sprite::Block),
	(core::EntityKind::Flippers, fx::Sprite::PowerFlippers),
	(core::EntityKind::FireBoots, fx::Sprite::PowerFireBoots),
	(core::EntityKind::IceSkates, fx::Sprite::PowerIceSkates),
	(core::EntityKind::SuctionBoots, fx::Sprite::PowerSuctionBoots),
	(core::EntityKind::BlueKey, fx::Sprite::BlueKey),
	(core::EntityKind::RedKey, fx::Sprite::RedKey),
	(core::EntityKind::GreenKey, fx::Sprite::GreenKey),
	(core::EntityKind::YellowKey, fx::Sprite::YellowKey),
	(core::EntityKind::Thief, fx::Sprite::Thief),
	(core::EntityKind::Bomb, fx::Sprite::Bomb),
	(core::EntityKind::Bug, fx::Sprite::BugUp),
	(core::EntityKind::FireBall, fx::Sprite::FireBall),
	(core::EntityKind::PinkBall, fx::Sprite::PinkBall),
	(core::EntityKind::Tank, fx::Sprite::TankUp),
	(core::EntityKind::Glider, fx::Sprite::GliderUp),
	(core::EntityKind::Teeth, fx::Sprite::TeethUp),
	(core::EntityKind::Walker, fx::Sprite::WalkerUpDown),
	(core::EntityKind::Blob, fx::Sprite::Blob),
	(core::EntityKind::Paramecium, fx::Sprite::ParameciumUpDown),
];

#[derive(Default)]
pub struct Input {
	pub left_click: bool,
	pub right_click: bool,
	pub key_left: bool,
	pub key_right: bool,
	pub key_up: bool,
	pub key_down: bool,
}

#[derive(Default)]
pub struct EditorState {
	pub game: fx::FxState,
	pub tool: Tool,

	pub screen_size: Vec2<i32>,
	pub cursor_pos: Vec2<i32>,
	pub mouse_pos: Vec3<f32>,

	pub selected_terrain: core::Terrain,
	pub selected_ent: core::EntityHandle,
	pub selected_args: Option<core::EntityArgs>,
	pub conn_src: Vec2<i32>,

	pub tool_pos: Option<Vec2<i32>>,
	pub input: Input,
}

impl EditorState {
	pub fn init(&mut self, resources: fx::Resources) {
		self.game.resources = resources;
		self.game.tiles = &tiles::TILES_EDIT;
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

		// Generate a new seed if zero, otherwise keep the existing seed
		let seed = if self.game.gs.field.seed == 0 {
			urandom::new().next_u32() as u64
		}
		else {
			self.game.gs.field.seed
		};

		let dto = core::FieldDto {
			name: self.game.gs.field.name.clone(),
			hint: self.game.gs.field.hint.clone(),
			password: self.game.gs.field.password.clone(),
			seed,
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
				self.tool_pos = Some(self.cursor_pos);
				match self.tool {
					Tool::Terrain => tool::terrain::think(self),
					Tool::Entity => tool::entity::think(self),
					Tool::Connection => tool::connection::think(self),
				}
			}
		}

		self.game.camera.eye_offset = Vec3::<f32>(0.0, 8.0 * 32.0, 400.0) * 2.0;
		self.game.camera.object_h = None;

		self.game.draw(g);

		g.begin().unwrap();

		let p = self.mouse_pos; {
			let mut cv = shade::d2::CommandBuffer::<fx::render::Vertex, fx::render::Uniform>::new();
			cv.shader = self.game.resources.shader;
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.viewport = cvmath::Rect::vec(self.screen_size);
			cv.push_uniform(fx::render::Uniform { transform: self.game.camera.view_proj_mat, texture: self.game.resources.tileset, texture_size: self.game.resources.tileset_size.map(|c| c as f32).into() });

			for y in 0..TERRAIN_SAMPLES.len() as i32 {
				for x in 0..2 {
					let terrain = TERRAIN_SAMPLES[y as usize][x as usize];
					let pos = Vec3::new((x - 3) as f32 * 32.0, y as f32 * 32.0, 0.0);
					fx::render::draw_tile(&mut cv, terrain, pos, &self.game.tiles);
				}
			}

			for i in 0..ENTITY_SAMPLES.len() as i32 {
				let (_, sprite) = ENTITY_SAMPLES[i as usize];
				let pos = Vec3::new(i as f32 * 32.0, -2.0 * 32.0, 0.0);
				fx::render::draw(&mut cv, pos, sprite, fx::Model::ReallyFlatSprite, 1.0);
			}

			match self.tool {
				Tool::Terrain => {
					fx::render::draw_tile(&mut cv, self.selected_terrain, p, &self.game.tiles);
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
			let mut cv = shade::d2::CommandBuffer::<fx::render::Vertex, fx::render::Uniform>::new();
			cv.shader = self.game.resources.shader;
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.viewport = cvmath::Rect::vec(self.screen_size);
			cv.push_uniform(fx::render::Uniform { transform: self.game.camera.view_proj_mat, texture: self.game.resources.tileset, texture_size: self.game.resources.tileset_size.map(|c| c as f32).into() });

			struct ToVertex {
				color: [u8; 4],
			}
			impl shade::d2::ToVertex<fx::render::Vertex> for ToVertex {
				fn to_vertex(&self, pos: Point2<f32>, _: usize) -> fx::render::Vertex {
					fx::render::Vertex { pos: pos.vec3(0.0), uv: Vec2::ZERO, color: self.color }
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
				if let Some(face_dir) = ent.face_dir {
					let pos = ent.pos.map(|c| c as f32 * 32.0 + 16.0);
					cv.draw_arrow(&pen, pos, pos + face_dir.to_vec().map(|c| c as f32 * 20.0), 4.0);
				}
			}
			cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
		}

		g.end().unwrap();
	}

	pub fn tool_terrain(&mut self, pressed: bool) {
		if pressed {
			self.tool = Tool::Terrain;
			self.selected_terrain = core::Terrain::Floor;
			self.tool_pos = None;
		}
	}
	pub fn tool_entity(&mut self, pressed: bool) {
		if pressed {
			self.tool = Tool::Entity;
			self.selected_ent = core::EntityHandle::INVALID;
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
				s.selected_ent = core::EntityHandle::INVALID;
				s.selected_args = Some(core::EntityArgs { kind, pos: cursor_pos, face_dir: None });
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
