use std::collections::HashMap;
use chipgame::fx::*;
use chipgame::core;
use cvmath::*;

#[derive(Clone, Default)]
pub struct EditorInput {
	pub mouse: Vec2<i32>,
	pub screen_size: Vec2<i32>,
	pub up: bool,
	pub left: bool,
	pub down: bool,
	pub right: bool,
	pub left_click: bool,
	pub right_click: bool,
}

#[derive(Clone, Debug)]
pub enum Tool {
	Terrain(core::Terrain),
	TerrainSampler,
	Entity(core::EntityArgs),
	EntitySampler,
	EntityMover,
	EntityEraser,
	Connector,
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
pub struct EditorGame {
	game: VisualState,
	input: EditorInput,
	pub tool: Tool,
	mover_args: Option<core::EntityArgs>,
	mover_pos: Vec2<i32>,
	tile_pos: Option<Vec2<i32>>,
	conn: core::Conn,
}

impl EditorGame {
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
	pub fn render(&mut self, g: &mut shade::Graphics, input: &EditorInput) {
		if input.left {
			self.game.camera.target.x -= 5.0;
		}
		if input.right {
			self.game.camera.target.x += 5.0;
		}
		if input.up {
			self.game.camera.target.y -= 5.0;
		}
		if input.down {
			self.game.camera.target.y += 5.0;
		}

		self.game.camera.eye_offset = Vec3::<f32>(0.0, 8.0 * 32.0, 400.0) * 2.0;
		self.game.camera.object_h = None;

		self.game.draw(g);

		let x = (input.mouse.x as f32 / input.screen_size.x as f32 - 0.5) * 2.0;
		let y = (input.mouse.y as f32 / input.screen_size.y as f32 - 0.5) * -2.0;

		// let pt_ndsh = Vec4::new(x, y, -1.0, 1.0);
		// let dir_eye = self.cam.proj_matrix.inverse() * pt_ndsh * 2.0;
		// // let dir_eye = dir_eye.with_w(0.0);
		// let dir_world = (self.cam.view_matrix.inverse() * dir_eye).xyz();
		// let dir = dir_world.normalize();

		// let inv = self.cam.view_proj_matrix.inverse();
		// let near = inv * Vec4::new(x, y, 0.0, 1.0);
		// let far = inv * Vec4::new(x, y, 1.0, 1.0);
		// let dir = (far.hdiv() - near.hdiv()).normalize();

		let x = x / self.game.camera.proj_mat.a11;
		let y = y / self.game.camera.proj_mat.a22;
		let dir = (self.game.camera.view_mat.inverse() * Vec4::new(x, y, -1.0, 1.0)).xyz().normalize();

		let ray = Ray::new(self.game.camera.target + self.game.camera.eye_offset, dir);
		let plane = Plane::new(Vec3::Z, 0.0);
		let mut hits = [TraceHit::default(); 2];
		let mut mouse_pos = None;
		let mut tile_pos = None;
		if ray.trace(&plane, &mut hits) > 0 {
			let p = ray.at(hits[0].distance);
			let pi = p.xy().map(|c| (c / 32.0).floor() as i32);
			if !self.input.left_click && input.left_click {
				println!("Click: [{},{}]", pi.x, pi.y);
			}
			mouse_pos = Some(p);
			tile_pos = Some(pi);
		}

		if let Some(tile_pos) = tile_pos {
			if self.input.left_click {
				if tile_pos.x < -1 {
					self.tool = Tool::TerrainSampler;
				}
				if tile_pos.y < -1 {
					self.tool = Tool::EntitySampler;
				}
			}
			match &self.tool {
				Tool::Terrain(terrain) => {
					if input.left_click {
						if self.tile_pos != Some(tile_pos) {
							self.game.gs.field.set_terrain(tile_pos, *terrain);
							self.tile_pos = Some(tile_pos);
						}
					}
					else {
						self.tile_pos = None;
					}
				}
				Tool::TerrainSampler => {
					if self.input.left_click && !input.left_click {
						if tile_pos.x < 0 {
							if tile_pos.x == -3 && tile_pos.y >= 0 && tile_pos.y < TERRAIN_SAMPLES.len() as i32 {
								self.tool = Tool::Terrain(TERRAIN_SAMPLES[tile_pos.y as usize][0]);
							}
							if tile_pos.x == -2 && tile_pos.y >= 0 && tile_pos.y < TERRAIN_SAMPLES.len() as i32 {
								self.tool = Tool::Terrain(TERRAIN_SAMPLES[tile_pos.y as usize][1]);
							}
						}
						else {
							let terrain = self.game.gs.field.get_terrain(tile_pos);
							self.tool = Tool::Terrain(terrain);
						}
					}
				}
				Tool::Entity(e) => {
					if input.left_click {
						if self.tile_pos != Some(tile_pos) {

							let handles = self.game.gs.ents.iter().filter_map(|ent| if ent.pos == tile_pos { Some(ent.handle) } else { None }).collect::<Vec<_>>();
							for ehandle in handles {
								self.game.gs.remove_entity(ehandle);
							}
							// self.game.sync();

							self.game.gs.create_entity(&core::EntityArgs { kind: e.kind, pos: tile_pos, face_dir: e.face_dir });
							self.game.sync();
							self.tile_pos = Some(tile_pos);
						}
					}
					else {
						self.tile_pos = None;
					}
				}
				Tool::EntitySampler => {
					if self.input.left_click && !input.left_click {
						if tile_pos.y < 0 {
							if tile_pos.y == -2 && tile_pos.x >= 0 && tile_pos.x < ENTITY_SAMPLES.len() as i32 {
								let (kind, _) = ENTITY_SAMPLES[tile_pos.x as usize];
								self.tool = Tool::Entity(core::EntityArgs { kind, pos: Vec2::ZERO, face_dir: None });
							}
						}
						else {
							let ehandle = self.game.gs.ents.iter().find_map(|ent| if ent.pos == tile_pos { Some(ent.handle) } else { None });
							if let Some(ehandle) = ehandle {
								if let Some(ent) = self.game.gs.ents.get(ehandle) {
									self.tool = Tool::Entity(ent.to_entity_args());
								}
							}
						}
					}
				}
				Tool::EntityEraser => {
					if self.input.left_click {
						let handles = self.game.gs.ents.iter().filter_map(|ent| if ent.pos == tile_pos { Some(ent.handle) } else { None }).collect::<Vec<_>>();
						for ehandle in handles {
							self.game.gs.remove_entity(ehandle);
						}
						self.game.sync();
					}
				}
				Tool::EntityMover => {
					if !self.input.left_click && input.left_click {
						let ehandle = self.game.gs.ents.iter().find_map(|ent| if ent.pos == tile_pos { Some(ent.handle) } else { None });
						if let Some(ehandle) = ehandle {
							if let Some(ent) = self.game.gs.ents.get(ehandle) {
								self.mover_args = Some(ent.to_entity_args());
							}
							self.game.gs.remove_entity(ehandle);
							// self.game.sync();
						}
					}
					if self.input.left_click && !input.left_click {
						if let Some(mover_args) = self.mover_args {
							self.game.gs.create_entity(&core::EntityArgs { kind: mover_args.kind, pos: tile_pos, face_dir: mover_args.face_dir });
							self.game.sync();
						}
					}
					if !self.input.right_click && input.right_click {
						self.mover_pos = tile_pos;
					}
					if self.input.right_click && !input.right_click {
						let mover_pos = self.mover_pos;
						let face_dir = match tile_pos - mover_pos {
							Vec2 { x: 0, y: -1 } => Some(core::Compass::Up),
							Vec2 { x: 0, y: 1 } => Some(core::Compass::Down),
							Vec2 { x: -1, y: 0 } => Some(core::Compass::Left),
							Vec2 { x: 1, y: 0 } => Some(core::Compass::Right),
							_ => None,
						};
						let ehandle = self.game.gs.ents.iter().find_map(|ent| if ent.pos == mover_pos { Some(ent.handle) } else { None });
						if let Some(ehandle) = ehandle {
							self.game.gs.set_entity_face_dir(ehandle, face_dir);
							self.game.sync();
						}
					}
				}
				Tool::Connector => {
					if !self.input.left_click && input.left_click {
						self.conn.src = tile_pos;
					}
					if self.input.left_click && !input.left_click {
						self.conn.dest = tile_pos;

						if self.conn.src != self.conn.dest {
							if let Some(index) = self.game.gs.field.conns.iter().position(|conn| conn == &self.conn) {
								self.game.gs.field.conns.remove(index);
							}
							else {
								self.game.gs.field.conns.push(self.conn);
							}
						}
					}
				}
			}
		}

		g.begin().unwrap();

		if let Some(p) = mouse_pos {
			let mut cv = shade::d2::Canvas::<render::Vertex, render::Uniform>::new();
			cv.shader = self.game.resources.shader;
			cv.depth_test = Some(shade::DepthTest::Less);
			cv.viewport = cvmath::Rect::vec(cvmath::Vec2(input.screen_size.x as i32, input.screen_size.y as i32));
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
			cv.viewport = cvmath::Rect::vec(cvmath::Vec2(input.screen_size.x as i32, input.screen_size.y as i32));
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

		self.input = input.clone();
	}
}
