use std::collections::HashMap;
use crate::visual::*;
use crate::core;
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
	pub chr: Option<char>,
}

// #[derive(Copy, Clone)]
enum Tool {
	Terrain(core::Terrain),
	Entity(core::EntityArgs),
	Erase,
}
impl Default for Tool {
	fn default() -> Self {
		Tool::Terrain(core::Terrain::Floor)
	}
}

#[derive(Default)]
pub struct EditorGame {
	game: VisualState,
	input: EditorInput,
	tool: Tool,
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

		match input.chr {
			Some('A') => self.tool = Tool::Terrain(core::Terrain::Blank),
			Some('B') => self.tool = Tool::Terrain(core::Terrain::Floor),
			Some('C') => self.tool = Tool::Terrain(core::Terrain::Wall),
			Some('D') => self.tool = Tool::Terrain(core::Terrain::BlueLock),
			Some('E') => self.tool = Tool::Terrain(core::Terrain::RedLock),
			Some('F') => self.tool = Tool::Terrain(core::Terrain::GreenLock),
			Some('G') => self.tool = Tool::Terrain(core::Terrain::YellowLock),
			Some('H') => self.tool = Tool::Terrain(core::Terrain::Hint),
			Some('I') => self.tool = Tool::Terrain(core::Terrain::Exit),
			Some('J') => self.tool = Tool::Terrain(core::Terrain::Water),
			Some('K') => self.tool = Tool::Terrain(core::Terrain::Fire),
			Some('L') => self.tool = Tool::Terrain(core::Terrain::Dirt),
			Some('M') => self.tool = Tool::Terrain(core::Terrain::Gravel),
			Some('N') => self.tool = Tool::Terrain(core::Terrain::Ice),
			Some('O') => self.tool = Tool::Terrain(core::Terrain::IceNW),
			Some('P') => self.tool = Tool::Terrain(core::Terrain::IceNE),
			Some('Q') => self.tool = Tool::Terrain(core::Terrain::IceSW),
			Some('R') => self.tool = Tool::Terrain(core::Terrain::IceSE),
			Some('S') => self.tool = Tool::Terrain(core::Terrain::ForceN),
			Some('T') => self.tool = Tool::Terrain(core::Terrain::ForceW),
			Some('U') => self.tool = Tool::Terrain(core::Terrain::ForceS),
			Some('V') => self.tool = Tool::Terrain(core::Terrain::ForceE),
			Some('W') => self.tool = Tool::Erase,
			Some('X') => self.tool = Tool::Entity(core::EntityArgs { kind: core::EntityKind::Block, pos: Vec2::ZERO, face_dir: None }),
			Some('Y') => self.tool = Tool::Entity(core::EntityArgs { kind: core::EntityKind::Chip, pos: Vec2::ZERO, face_dir: None }),
			Some('Z') => self.tool = Tool::Entity(core::EntityArgs { kind: core::EntityKind::Socket, pos: Vec2::ZERO, face_dir: None }),
			_ => (),
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
			let pi = p.xy().map(|c| (c / 32.0) as i32);
			if !self.input.left_click && input.left_click {
				println!("Click: [{},{}]", pi.x, pi.y);
			}
			mouse_pos = Some(p);
			tile_pos = Some(pi);
		}

		if input.left_click {
			if self.tile_pos != tile_pos {
				if let Some(tile_pos) = tile_pos {
					match &self.tool {
						&Tool::Terrain(terrain) => {
							self.game.gs.field.set_terrain(tile_pos, terrain);
						}
						Tool::Entity(e) => {
							self.game.gs.create_entity(&core::EntityArgs { kind: e.kind, pos: tile_pos, face_dir: e.face_dir });
							self.game.sync();
						}
						Tool::Erase => {
							let handles = self.game.gs.ents.iter().filter_map(|ent| if ent.pos == tile_pos { Some(ent.handle) } else { None }).collect::<Vec<_>>();
							for ehandle in handles {
								self.game.gs.remove_entity(ehandle);
							}
							self.game.sync();
						}
					}
				}
				self.tile_pos = tile_pos;
			}
		}
		else {
			self.tile_pos = None;
		}

		if !self.input.right_click && input.right_click {
			if let Some(tile_pos) = tile_pos {
				self.conn.src = tile_pos;
			}
		}
		if self.input.right_click && !input.right_click {
			if let Some(tile_pos) = tile_pos {
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

			struct ToVertex;
			impl shade::d2::ToVertex<render::Vertex> for ToVertex {
				fn to_vertex(&self, pos: Point2<f32>, _: usize) -> render::Vertex {
					render::Vertex { pos: pos.vec3(0.0), uv: Vec2::ZERO, color: [0, 0, 255, 255] }
				}
			}
			for conn in &self.game.gs.field.conns {
				let pen = shade::d2::Pen { template: ToVertex, segments: 0 };
				let src = conn.src.map(|c| c as f32 * 32.0 + 16.0);
				let dest = conn.dest.map(|c| c as f32 * 32.0 + 16.0);
				cv.draw_arrow(&pen, src, dest, 12.0);
			}
			cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
		}

		g.end().unwrap();

		self.input = input.clone();
	}
}
