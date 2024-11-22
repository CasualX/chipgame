use super::*;

pub struct DrawPlayTitle<'a> {
	pub level_number: i32,
	pub level_name: &'a str,
	pub subtitle: Option<&'a dyn fmt::Display>,
}

impl<'a> DrawPlayTitle<'a> {
	pub fn draw(&self, buf: &mut shade::d2::TextBuffer, rect: &Rect<f32>, resx: &Resources) {
		let size = resx.screen_size.y as f32 * FONT_SIZE;

		let scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		if let Some(subtitle) = self.subtitle {
			buf.text_lines(&resx.font, &scribe, rect, shade::d2::BoxAlign::MiddleCenter, &[
				&format_args!("~ Level {} ~", self.level_number),
				&format_args!("\x1b[color=#ff0]{}", self.level_name),
				subtitle,
			]);
		}
		else {
			buf.text_lines(&resx.font, &scribe, rect, shade::d2::BoxAlign::MiddleCenter, &[
				&format_args!("~ Level {} ~", self.level_number),
				&format_args!("\x1b[color=#ff0]{}", self.level_name),
			]);
		}
	}
}
