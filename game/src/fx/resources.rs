use super::*;

#[derive(Default)]
pub struct Resources {
	pub tileset: shade::Texture2D,
	pub tileset_size: Vec2<i32>,
	pub shader: shade::Shader,
	pub screen_size: Vec2<i32>,

	pub uishader: shade::Shader,
	pub texdigits: shade::Texture2D,

	pub font: shade::d2::FontResource<Option<shade::msdfgen::Font>>,
}
