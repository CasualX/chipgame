use super::*;

pub struct DrawMenuItems<'a> {
	pub items_text: &'a [&'a dyn fmt::Display],
	pub selected_index: usize,
}

impl<'a> DrawMenuItems<'a> {
	pub fn draw(&self, buf: &mut shade::d2::TextBuffer, rect: &cvmath::Rect<f32>, resx: &Resources) {
		let size = resx.screen_size.y as f32 * FONT_SIZE;

		let mut scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		let y = (rect.height() - self.items_text.len() as f32 * scribe.line_height) * 0.5;

		for (i, &line) in self.items_text.iter().enumerate() {
			let color = if i == self.selected_index { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;

			let rect = cvmath::Rect::c(rect.mins.x, rect.mins.y + y + i as f32 * scribe.line_height, rect.maxs.x, rect.mins.y + y + (i + 1) as f32 * scribe.line_height);
			buf.text_fmt_lines(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, &[format_args!("{}", line)]);
		}
	}
}
