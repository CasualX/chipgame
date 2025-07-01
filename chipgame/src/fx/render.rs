use super::*;

// Vertex definition

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct Vertex {
	pub pos: Vec3<f32>,
	pub uv: Vec2<f32>,
	pub color: [u8; 4],
}

unsafe impl shade::TVertex for Vertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: std::mem::size_of::<Vertex>() as u16,
		alignment: std::mem::align_of::<Vertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "a_pos",
				format: shade::VertexAttributeFormat::F32v3,
				offset: dataview::offset_of!(Vertex.pos) as u16,
			},
			shade::VertexAttribute {
				name: "a_texcoord",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(Vertex.uv) as u16,
			},
			shade::VertexAttribute {
				name: "a_color",
				format: shade::VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(Vertex.color) as u16,
			},
		],
	};
}

// Uniform definition

#[derive(Clone, Debug, PartialEq)]
pub struct Uniform {
	pub transform: Mat4<f32>,
	pub texture: shade::Texture2D,
}

impl Default for Uniform {
	fn default() -> Self {
		Uniform {
			transform: Mat4::IDENTITY,
			texture: shade::Texture2D::INVALID,
		}
	}
}

impl shade::UniformVisitor for Uniform {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("transform", &self.transform);
		set.value("tex", &self.texture);
	}
}

const T_S: f32 = 32.0; // TILE SIZE

pub struct ModelData {
	pub vertices: &'static [Vertex],
	pub indices: &'static [u32],
}

impl ModelData {
	pub const FLOOR: ModelData = ModelData {
		vertices: &[
			Vertex { pos: Vec3(0.0, 0.0, 0.0), uv: Vec2(0.0, 0.0), color: [255, 255, 255, 255] },
			Vertex { pos: Vec3(0.0, T_S, 0.0), uv: Vec2(0.0, 1.0), color: [255, 255, 255, 255] },
			Vertex { pos: Vec3(T_S, T_S, 0.0), uv: Vec2(1.0, 1.0), color: [255, 255, 255, 255] },
			Vertex { pos: Vec3(T_S, 0.0, 0.0), uv: Vec2(1.0, 0.0), color: [255, 255, 255, 255] },
		],
		indices: &[0, 1, 2, 0, 2, 3],
	};

	pub const WALL: ModelData = {
		const S: f32 = 4.0;
		const H: f32 = 20.0;
		ModelData {
			vertices: &[
				Vertex { pos: Vec3(0.0, 0.0, 0.0), uv: Vec2(0.0, 0.0), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(0.0, T_S, 0.0), uv: Vec2(0.0, 1.0), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(T_S, T_S, 0.0), uv: Vec2(1.0, 1.0), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(T_S, 0.0, 0.0), uv: Vec2(1.0, 0.0), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(      S,       S,   H), uv: Vec2(0.0, 0.1), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(      S, T_S - S,   H), uv: Vec2(0.0, 0.9), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(T_S - S, T_S - S,   H), uv: Vec2(0.9, 0.9), color: [255, 255, 255, 255] },
				Vertex { pos: Vec3(T_S - S,       S,   H), uv: Vec2(0.9, 0.1), color: [255, 255, 255, 255] },
			],
			indices: &[
				0, 1, 4, 4, 1, 5,
				1, 2, 5, 5, 2, 6,
				2, 3, 6, 6, 3, 7,
				3, 0, 7, 7, 0, 4,
				4, 6, 7, 4, 5, 6,
			],
		}
	};

	pub const SPRITE: ModelData = ModelData {
		vertices: &[
			Vertex { pos: Vec3(0.0, 0.0, 20.0), uv: Vec2(0.0, 0.0), color: [255, 255, 255, 255] },
			Vertex { pos: Vec3(0.0, T_S, 20.0), uv: Vec2(0.0, 1.0), color: [255, 255, 255, 255] },
			Vertex { pos: Vec3(T_S, T_S, 20.0), uv: Vec2(1.0, 1.0), color: [255, 255, 255, 255] },
			Vertex { pos: Vec3(T_S, 0.0, 20.0), uv: Vec2(1.0, 0.0), color: [255, 255, 255, 255] },
		],
		indices: &[0, 1, 2, 0, 2, 3],
	};

	pub const SPRITE_SHADOW: ModelData = ModelData {
		vertices: &[
			Vertex { pos: Vec3(0.0, 0.0, 0.5), uv: Vec2(0.0, 0.0), color: [0, 0, 0, 128] },
			Vertex { pos: Vec3(0.0, T_S, 0.5), uv: Vec2(0.0, 1.0), color: [0, 0, 0, 128] },
			Vertex { pos: Vec3(T_S, T_S, 0.5), uv: Vec2(1.0, 1.0), color: [0, 0, 0, 128] },
			Vertex { pos: Vec3(T_S, 0.0, 0.5), uv: Vec2(1.0, 0.0), color: [0, 0, 0, 128] },
		],
		indices: &[0, 1, 2, 0, 2, 3],
	};
}

const TILE_SIZE: f32 = 32.0;

fn draw_floor(cv: &mut shade::d2::DrawBuilder<Vertex, Uniform>, pos: Vec3<f32>, sprite: data::SpriteId, z1: f32, z2: f32, alpha: f32) {
	let gfx = sprite.index();

	let mut p = cv.begin(shade::PrimType::Triangles, 4, 2);
	p.add_indices_quad();

	let x = pos.x;
	let y = pos.y;
	let z1 = z1 + pos.z;
	let z2 = z2 + pos.z;
	let a = (alpha * 255.0) as u8;

	let u = gfx.x as f32 * (TILE_SIZE + 2.0) + 1.0;
	let v = gfx.y as f32 * (TILE_SIZE + 2.0) + 1.0;

	p.add_vertex(Vertex {
		pos: Vec3(x, y, z2),
		uv: Vec2(u, v),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x, y + TILE_SIZE, z1),
		uv: Vec2(u, v + TILE_SIZE),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE, y + TILE_SIZE, z1),
		uv: Vec2(u + TILE_SIZE, v + TILE_SIZE),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE, y, z2),
		uv: Vec2(u + TILE_SIZE, v),
		color: [255, 255, 255, a],
	});
}

fn draw_shadow(cv: &mut shade::d2::DrawBuilder<Vertex, Uniform>, pos: Vec3<f32>, sprite: data::SpriteId, skew: f32, a: f32) {
	let gfx = sprite.index();

	let mut p = cv.begin(shade::PrimType::Triangles, 4, 2);
	p.add_indices_quad();

	let x = pos.x;
	let y = pos.y;
	let s = skew;

	let u = gfx.x as f32 * (TILE_SIZE + 2.0) + 1.0;
	let v = gfx.y as f32 * (TILE_SIZE + 2.0) + 1.0;
	let a = (a * 128.0) as u8;

	p.add_vertex(Vertex {
		pos: Vec3(x + s, y, 0.5),
		uv: Vec2(u, v),
		color: [0, 0, 0, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x, y + TILE_SIZE, 0.5),
		uv: Vec2(u, v + TILE_SIZE),
		color: [0, 0, 0, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE, y + TILE_SIZE, 0.5),
		uv: Vec2(u + TILE_SIZE, v + TILE_SIZE),
		color: [0, 0, 0, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + s + TILE_SIZE, y, 0.5),
		uv: Vec2(u + TILE_SIZE, v),
		color: [0, 0, 0, a],
	});
}

fn draw_wall(cv: &mut shade::d2::DrawBuilder<Vertex, Uniform>, pos: Vec3<f32>, w: f32, sprite: data::SpriteId, alpha: f32) {
	let gfx = sprite.index();

	let mut p = cv.begin(shade::PrimType::Triangles, 8, 10);

	p.add_indices(&[
		0, 1, 4, 4, 1, 5,
		1, 2, 5, 5, 2, 6,
		2, 3, 6, 6, 3, 7,
		3, 0, 7, 7, 0, 4,
		4, 6, 7, 4, 5, 6,
	]);

	let x = pos.x;
	let y = pos.y;
	let z = pos.z;
	let a = (alpha * 255.0) as u8;

	let u = gfx.x as f32 * (TILE_SIZE + 2.0) + 1.0;
	let v = gfx.y as f32 * (TILE_SIZE + 2.0) + 1.0;

	let s = 4.0 + w;//if matches!(sprite, data::Sprite::Wall) { 0.0 } else { 4.0 };
	let t = 4.0;
	let h = 20.0; //if block.is_door() { 15.0 } else { 20.0 };

	p.add_vertex(Vertex {
		pos: Vec3(x + w, y + w, z),
		uv: Vec2(u, v),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + w, y + TILE_SIZE - w, z),
		uv: Vec2(u, v + TILE_SIZE),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE - w, y + TILE_SIZE - w, z),
		uv: Vec2(u + TILE_SIZE, v + TILE_SIZE),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE - w, y + w, z),
		uv: Vec2(u + TILE_SIZE, v),
		color: [255, 255, 255, a],
	});

	p.add_vertex(Vertex {
		pos: Vec3(x + s, y + s, z + h),
		uv: Vec2(u + t, v + t),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + s, y + TILE_SIZE - s, z + h),
		uv: Vec2(u + t, v + TILE_SIZE - t),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE - s, y + TILE_SIZE - s, z + h),
		uv: Vec2(u + TILE_SIZE - t, v + TILE_SIZE - t),
		color: [255, 255, 255, a],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE - s, y + s, z + h),
		uv: Vec2(u + TILE_SIZE - t, v + t),
		color: [255, 255, 255, a],
	});
}

fn draw_portal(cv: &mut shade::d2::DrawBuilder<Vertex, Uniform>, pos: Vec3<f32>, sprite: data::SpriteId) {
	let gfx = sprite.index();

	let mut p = cv.begin(shade::PrimType::Triangles, 5, 4);
	p.add_indices(&[0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1]);

	let x = pos.x;
	let y = pos.y;
	let z = pos.z;

	let cx = x + TILE_SIZE * 0.5;
	let cy = y + TILE_SIZE * 0.5;

	let u = gfx.x as f32 * (TILE_SIZE + 2.0) + 1.0;
	let v = gfx.y as f32 * (TILE_SIZE + 2.0) + 1.0;
	let cu = u + TILE_SIZE * 0.5;
	let cv = v + TILE_SIZE * 0.5;

	p.add_vertex(Vertex {
		pos: Vec3(cx, cy, z - 10.0),
		uv: Vec2(cu, cv),
		color: [255, 255, 255, 255],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x, y, z),
		uv: Vec2(u, v),
		color: [255, 255, 255, 255],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x, y + TILE_SIZE, z),
		uv: Vec2(u, v + TILE_SIZE),
		color: [255, 255, 255, 255],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE, y + TILE_SIZE, z),
		uv: Vec2(u + TILE_SIZE, v + TILE_SIZE),
		color: [255, 255, 255, 255],
	});
	p.add_vertex(Vertex {
		pos: Vec3(x + TILE_SIZE, y, z),
		uv: Vec2(u + TILE_SIZE, v),
		color: [255, 255, 255, 255],
	});
}

pub fn draw(cv: &mut shade::d2::DrawBuilder<Vertex, Uniform>, pos: Vec3<f32>, sprite: data::SpriteId, model: data::ModelId, alpha: f32) {
	match model {
		data::ModelId::Empty => (),
		data::ModelId::Floor => draw_floor(cv, pos, sprite, 0.0, 0.0, alpha),
		data::ModelId::Wall => draw_wall(cv, pos, 0.0, sprite, alpha),
		data::ModelId::ThinWall => draw_wall(cv, pos, 2.0, sprite, alpha),
		data::ModelId::Sprite => draw_floor(cv, pos, sprite, 0.0, 20.0, alpha),
		data::ModelId::Portal => draw_portal(cv, pos, sprite),
		data::ModelId::FlatSprite => draw_floor(cv, pos, sprite, 3.0, 12.0, alpha),
		data::ModelId::ReallyFlatSprite => draw_floor(cv, pos, sprite, 6.0, 10.0, alpha),
		data::ModelId::FloorSprite => draw_floor(cv, pos, sprite, 1.0, 1.0, alpha),
		_ => unimplemented!(),
	}
}

pub fn draw_tile(cv: &mut shade::d2::DrawBuilder::<render::Vertex, render::Uniform>, terrain: core::Terrain, pos: Vec3<f32>, tiles: &[TileGfx]) {
	let tile = tiles[terrain as usize];
	draw(cv, pos, tile.sprite, tile.model, 1.0);
}

fn is_block_on_pos(state: &FxState, pos: Vec2<i32>) -> bool {
	for h in state.gs.qt.get(pos) {
		if let Some(ent) = state.gs.ents.get(h) {
			if matches!(ent.kind, core::EntityKind::Block) {
				return true;
			}
		}
	}
	false
}

pub fn field(cv: &mut shade::d2::DrawBuilder::<render::Vertex, render::Uniform>, state: &FxState, time: f32) {
	let i = (time * 8.0) as i32;
	let field = &state.gs.field;
	// let resx = &state.resources;
	// Render the level geometry
	cv.blend_mode = shade::BlendMode::Solid;
	for y in 0..field.height {
		for x in 0..field.width {
			let tile = field.get_terrain(Vec2(x, y));
			let tile = state.tiles[tile as usize];
			if tile.sprite == data::SpriteId::Blank || tile.model == data::ModelId::Empty {
				continue;
			}
			let (mut sprite, model) = (tile.sprite, tile.model);
			if tile.sprite == data::SpriteId::Exit1 {
				match i % 3 {
					2 => sprite = data::SpriteId::Exit1,
					1 => sprite = data::SpriteId::Exit2,
					0 => sprite = data::SpriteId::Exit3,
					_ => (),
				}
			}
			// Make Blocks appear on top of walls
			let z = if matches!(model, data::ModelId::Wall) && is_block_on_pos(state, Vec2(x, y)) { -2.0 } else { 0.0 };
			draw(cv, Vec3(x, y, 0).map(|c| c as f32 * 32.0).set_z(z), sprite, model, 1.0);
		}
	}
	// Render the object shadows
	cv.blend_mode = shade::BlendMode::Alpha;
	for obj in state.objects.map.values() {
		if !obj.live || !obj.vis {
			continue;
		}
		if matches!(obj.model, data::ModelId::Sprite | data::ModelId::FlatSprite) {
			draw_shadow(cv, obj.pos, obj.sprite, 10.0, obj.alpha);
		}
		if matches!(obj.model, data::ModelId::ReallyFlatSprite) {
			draw_shadow(cv, obj.pos, obj.sprite, 2.0, obj.alpha);
		}
	}
	// Render the objects
	for obj in state.objects.map.values() {
		if !obj.live || !obj.vis {
			continue;
		}
		draw(cv, obj.pos, obj.sprite, obj.model, obj.alpha);
	}
}
