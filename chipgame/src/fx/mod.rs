/*!
Presentation layer
==================
 */

use std::collections::HashMap;
use cvmath::*;

mod camera;
mod event;
mod handlers;
mod object;
mod objectmap;
mod fxstate;
mod resources;
pub mod render;
mod tile;
mod hud;

pub use self::camera::*;
pub use self::event::*;
pub use self::handlers::*;
pub use self::object::*;
pub use self::objectmap::*;
pub use self::fxstate::*;
pub use self::resources::*;
pub use self::render::*;
pub use self::tile::*;

use crate::data;
use crate::menu::Input;

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

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum HighScore {
	Record,
	Tied,
	#[default]
	None,
}



#[track_caller]
fn load_png(
	g: &mut shade::Graphics,
	name: Option<&str>,
	fs: &crate::FileSystem,
	path: &str,
	props: &shade::image::TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut shade::image::ImageSize)>,
) -> Result<shade::Texture2D, shade::image::png::LoadError> {
	let data = fs.read(path).expect("Failed to read PNG file");
	shade::image::png::load_stream(g, name, &mut &data[..], props, transform)
}

pub fn load_graphics(fs: &crate::FileSystem, config: &crate::config::Config, g: &mut shade::Graphics, resx: &mut Resources) {
	for (name, shader) in &config.shaders {
		let vs = fs.read_to_string(&shader.vertex_shader).expect("Failed to read shader vertex file");
		let fs = fs.read_to_string(&shader.fragment_shader).expect("Failed to read shader fragment file");
		g.shader_create(Some(name.as_str()), &vs, &fs);
	}
	for (name, texture) in &config.textures {
		let transform = match texture.transform {
			Some(crate::config::TransformType::Gutter32x32) => Some(&mut shade::image::gutter(32, 32) as _),
			_ => None,
		};
		load_png(g, Some(name.as_str()), fs, &texture.path, &texture.props, transform).expect("Failed to load texture");
	}
	let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS);
	for (name, font_config) in &config.fonts {
		let font = fs.read_to_string(&font_config.meta).expect("Failed to read font meta file");
		let font: shade::msdfgen::FontDto = serde_json::from_str(&font).expect("Failed to parse font meta file");
		let font: Option<shade::msdfgen::Font> = Some(font.into());
		let texture = load_png(g, Some(name.as_str()), fs, &font_config.atlas, &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).expect("Failed to load font atlas");
		let font = shade::d2::FontResource { font, shader, texture };
		resx.font = font;
	}

	resx.effects = g.texture2d_find("Effects");
	resx.tileset = g.texture2d_find("Tileset");
	resx.tileset_size = {
		let info = g.texture2d_get_info(resx.tileset);
		[info.width, info.height].into()
	};
	resx.shader = g.shader_find("PixelArt");
	resx.pixel_art_bias = config.pixel_art_bias;
	resx.colorshader = g.shader_find("Color");
	resx.uishader = g.shader_find("UI");
	resx.menubg = g.texture2d_find("MenuBG");
	resx.menubg_scale = 2.0;
}
