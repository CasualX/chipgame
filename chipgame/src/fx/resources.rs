use super::*;

#[derive(Default)]
pub struct Resources {
	pub effects: shade::Texture2D,
	pub tileset: shade::Texture2D,
	pub tileset_size: Vec2<i32>,
	pub shader: shade::Shader,
	pub viewport: Bounds2i,

	pub colorshader: shade::Shader,
	pub uishader: shade::Shader,
	pub texdigits: shade::Texture2D,
	pub menubg: shade::Texture2D,
	pub menubg_scale: f32,

	pub font: shade::d2::FontResource<Option<shade::msdfgen::Font>>,
}
