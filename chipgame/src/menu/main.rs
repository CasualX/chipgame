use super::*;

#[derive(Default)]
pub struct MainMenu {
	pub selected: u8,
}

impl MainMenu {
	const ITEMS: [&'static str; 7] = ["New game", "Continue", "Go to level", "High scores", "Options", "About", "Exit"];
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			self.selected = if self.selected > 0 { self.selected - 1 } else { self.selected };
		}
		if input.down.is_pressed() {
			self.selected = if self.selected < Self::ITEMS.len() as u8 - 1 { self.selected + 1 } else { self.selected };
		}
		if input.a.is_pressed() || input.start.is_pressed() {
			let evt = match self.selected {
				0 => MenuEvent::NewGame,
				1 => MenuEvent::Continue,
				2 => MenuEvent::LevelSelect,
				3 => MenuEvent::HighScores,
				4 => MenuEvent::Options,
				5 => MenuEvent::About,
				_ => MenuEvent::Exit,
				// _ => MenuEvent::None,
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

		let size = resx.screen_size.y as f32 * FONT_SIZE;

		// let mut pos = Vec2::ZERO;
		let mut scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			..Default::default()
		};

		for (i, item) in Self::ITEMS.iter().enumerate() {
			let color = if i == self.selected as usize { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 * 0.5 - 100.0 + i as i32 as f32 * scribe.line_height));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, item);
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
