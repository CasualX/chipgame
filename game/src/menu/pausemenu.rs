use super::*;

#[derive(Default)]
pub struct PauseMenu {
	selected: u8,
	pub events: Vec<MenuEvent>,
}

impl PauseMenu {
	pub fn think(&mut self, input: &core::Input) {

	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		darken(g, resx, 128);

		// let mut buf = shade::d2::TextBuffer::new();
		// buf.shader = resx.font.shader;
		// buf.blend_mode = shade::BlendMode::Alpha;
		// buf.viewport = cvmath::Rect::vec(resx.screen_size);

		// let ss = resx.screen_size;
		// let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));

		// buf.push_uniform(shade::d2::TextUniform {
		// 	transform,
		// 	texture: resx.font.texture,
		// 	..Default::default()
		// });

		// let size = resx.screen_size.y as f32 / 20.0;

		// let mut scribe = shade::d2::Scribe {
		// 	buf,
		// 	pos: Vec2f::new(0.0, 0.0),
		// 	size,
		// 	color: [255, 255, 255, 255],
		// };

		// scribe.write("PAUSED", shade::d2::TextAlign::Center);
	}
}
