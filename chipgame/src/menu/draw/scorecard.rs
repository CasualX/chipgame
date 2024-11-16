use super::*;

pub struct DrawScoreCard {
	pub attempts: i32,
	pub time: i32,
	pub steps: i32,
	pub bonks: i32,
}
impl DrawScoreCard {
	pub fn draw(&self, buf: &mut shade::d2::TextBuffer, rect: &Rect<f32>, resx: &Resources) {
		let size = resx.screen_size.y as f32 * FONT_SIZE;

		let mut scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		scribe.color = cvmath::Vec4(255, 255, 255, 255);
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleLeft, "Attempts:\nTime:\nSteps:\nBonks:");

		scribe.color = cvmath::Vec4(0, 255, 128, 255);
		let frames = self.time % 60;
		let seconds = (self.time / 60) % 60;
		let minutes = self.time / 3600;
		if minutes > 0 {
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleRight, &format!("{}\n{}:{:02}.{:02}\n{}\n{}", self.attempts, minutes, seconds, frames, self.steps, self.bonks));
		}
		else {
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleRight, &format!("{}\n{}.{:02}\n{}\n{}", self.attempts, seconds, frames, self.steps, self.bonks));
		}
	}
}