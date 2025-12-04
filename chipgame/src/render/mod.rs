//! Renderer.

use std::collections::HashMap;
use cvmath::*;

use crate::fx::Resources;

mod animation;
mod effect;
mod object;
mod objectmap;
mod render;
mod renderstate;

pub use self::animation::*;
pub use self::effect::*;
pub use self::object::*;
pub use self::objectmap::*;
pub use self::render::*;
pub use self::renderstate::*;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TileGfx {
	pub sprite: chipty::SpriteId,
	pub model: chipty::ModelId,
}

pub fn drawbg(g: &mut shade::Graphics, resx: &Resources) {
	let mut cv = shade::d2::DrawBuilder::<render::Vertex, render::Uniform>::new();
	cv.viewport = resx.viewport;
	cv.depth_test = None;
	cv.cull_mode = None;
	cv.shader = resx.shader;
	cv.uniform.texture = resx.menubg;
	cv.uniform.pixel_bias = resx.pixel_art_bias;
	let info = g.texture2d_get_info(resx.menubg);
	let tex_w = info.width as f32;
	let tex_h = info.height as f32;
	let vp_w = resx.viewport.width() as f32;
	let vp_h = resx.viewport.height() as f32;
	// Number of times the texture should repeat across the screen.
	let repeat_x = vp_w / (tex_w * resx.menubg_scale);
	let repeat_y = vp_h / (tex_h * resx.menubg_scale);
	// In pixel units (vertex shader divides by texture size)
	let u_max = tex_w * repeat_x;
	let v_max = tex_h * repeat_y;
	{
		let mut p = cv.begin(shade::PrimType::Triangles, 4, 2);
		p.add_indices_quad();
		p.add_vertices(&[
			// Note: Y flipped like original (bottom uses v_max, top uses 0.0)
			render::Vertex { pos: cvmath::Vec3(-1.0, -1.0, 0.0), uv: cvmath::Vec2(0.0, v_max), color: [255; 4] },
			render::Vertex { pos: cvmath::Vec3( 1.0, -1.0, 0.0), uv: cvmath::Vec2(u_max, v_max), color: [255; 4] },
			render::Vertex { pos: cvmath::Vec3( 1.0,  1.0, 0.0), uv: cvmath::Vec2(u_max, 0.0),  color: [255; 4] },
			render::Vertex { pos: cvmath::Vec3(-1.0,  1.0, 0.0), uv: cvmath::Vec2(0.0, 0.0),   color: [255; 4] },
		]);
	}
	cv.draw(g, shade::Surface::BACK_BUFFER);
	g.clear(&shade::ClearArgs { surface: shade::Surface::BACK_BUFFER, depth: Some(1.0), ..Default::default() });
}

pub fn sprite_uv(sheet: &chipty::SpriteSheet<chipty::SpriteId>, sprite: chipty::SpriteId, frame: usize) -> Vec2f {
	let Some(entry) = sheet.sprites.get(&sprite) else {
		panic!("sprite {:?} not found in sheet", sprite);
	};
	let index = entry.index as usize + if entry.len == 0 {
		panic!("sprite entry has zero frames");
	}
	else if entry.len == 1 {
		0
	}
	else {
		frame % (entry.len as usize)
	};
	let f = &sheet.frames[index];
	Vec2(f.rect[0] as f32, f.rect[1] as f32)
}
