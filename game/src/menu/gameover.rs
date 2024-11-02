use super::*;

#[derive(Default)]
pub struct GameOverMenu {
	pub selected: u8,
	pub level_index: i32,
	pub level_name: String,
	pub attempts: i32,
	pub time: i32,
	pub steps: i32,
	pub bonks: i32,
}

impl GameOverMenu {
	const ITEMS: [&'static str; 2] = ["Retry", "Main menu"];
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			if self.selected > 0 {
				self.selected -= 1;
				events.push(MenuEvent::CursorMove);
			}
		}
		if input.down.is_pressed() {
			if self.selected < Self::ITEMS.len() as u8 - 1 {
				self.selected += 1;
				events.push(MenuEvent::CursorMove);
			}
		}
		if input.a.is_pressed() || input.start.is_pressed() {
			let evt = match self.selected {
				0 => MenuEvent::Retry,
				_ => MenuEvent::MainMenu,
			};
			events.push(evt);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		let mut buf = shade::d2::TextBuffer::new();
		buf.shader = resx.font.shader;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.viewport = cvmath::Rect::vec(resx.screen_size);

		let ss = resx.screen_size;
		let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));

		buf.push_uniform(shade::d2::TextUniform {
			transform,
			texture: resx.font.texture,
			..Default::default()
		});

		let size = resx.screen_size.y as f32 / 20.0;

		let mut scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, size * 3.0));
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, &format!("Level {}: {}", self.level_index, self.level_name));
		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, size * 3.0 + scribe.line_height));
		scribe.color = cvmath::Vec4(255, 0, 128, 255);
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, "Game Over!");

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5 - size * 4.0, resx.screen_size.y as f32 * 0.5));
		scribe.color = cvmath::Vec4(255, 255, 255, 255);
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleLeft, "Attempts:\nTime:\nSteps:\nBonks:");

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5 + size * 4.0, resx.screen_size.y as f32 * 0.5));
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

		// let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 - size * 3.0));
		// scribe.color = cvmath::Vec4(255, 255, 255, 255);
		// buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::BottomCenter, "Next level\nRetry\nMain menu");
		for (i, item) in Self::ITEMS.iter().enumerate() {
			let color = if i == self.selected as usize { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 - size * (2.0 + Self::ITEMS.len() as f32) + i as i32 as f32 * scribe.line_height));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, item);
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
