use super::*;

#[derive(Default)]
pub struct ScoutMode {
}

impl ScoutMode {
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.select.is_pressed() {
			events.push(MenuEvent::ResumePlay);
		}
		if input.start.is_pressed() {
			events.push(MenuEvent::CloseMenu);
			events.push(MenuEvent::OpenPauseMenu);
		}

		if input.up.is_held() {
			events.push(MenuEvent::ScoutN);
		}
		if input.down.is_held() {
			events.push(MenuEvent::ScoutS);
		}
		if input.left.is_held() {
			events.push(MenuEvent::ScoutW);
		}
		if input.right.is_held() {
			events.push(MenuEvent::ScoutE);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		let mut buf = shade::d2::TextBuffer::new();
		buf.viewport = resx.viewport;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.shader = resx.font.shader;

		let rect = resx.viewport.cast();
		buf.uniform.transform = Transform2f::ortho(rect);
		buf.uniform.texture = resx.font.texture;

		let size = rect.height() * FONT_SIZE * 0.75;

		let scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: Vec4(255, 255, 255, 255),
			..Default::default()
		};

		// buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleCenter, "Paused");
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::TopCenter, "^");
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::BottomCenter, "v");
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleLeft, " <");
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleRight, "> ");

		buf.draw(g, shade::Surface::BACK_BUFFER);
	}
}
