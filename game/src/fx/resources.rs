use super::*;

#[derive(Default)]
pub struct Resources {
	pub tileset: shade::Texture2D,
	pub tileset_size: Vec2<i32>,
	pub shader: shade::Shader,
	pub screen_size: Vec2<i32>,
}
