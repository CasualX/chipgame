use super::*;

#[derive(Default)]
pub struct MainMenu {
	pub title: String,
	pub selected: u8,
}

impl MainMenu {
	const ITEMS: [&'static str; 7] = ["New game", "Continue", "Go to level", "High scores", "Options", "About", "Exit"];
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			if self.selected > 0 {
				self.selected = self.selected - 1;
				events.push(MenuEvent::CursorMove);
			}
		}
		if input.down.is_pressed() {
			if self.selected < Self::ITEMS.len() as u8 - 1 {
				self.selected = self.selected + 1;
				events.push(MenuEvent::CursorMove);
			}
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
			events.push(MenuEvent::CursorSelect);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		let mut buf = shade::d2::TextBuffer::new();
		buf.shader = resx.font.shader;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.viewport = cvmath::Rect::vec(resx.screen_size);

		let rect = Rect::vec(resx.screen_size.cast::<f32>());
		let transform = foo(rect, Rect::c(-1.0, 1.0, 1.0, -1.0));

		buf.push_uniform(shade::d2::TextUniform {
			transform,
			texture: resx.font.texture,
			..Default::default()
		});

		let [top, bottom, _] = draw::flexv(rect, None, layout::Justify::Center, &[layout::Unit::Fr(1.0), layout::Unit::Fr(3.0), layout::Unit::Fr(1.0)]);

		{
			let size = resx.screen_size.y as f32 * FONT_SIZE;

			let scribe = shade::d2::Scribe {
				font_size: size,
				line_height: size * (5.0 / 4.0),
				color: cvmath::Vec4(255, 255, 255, 255),
				..Default::default()
			};

			buf.text_box(&resx.font, &scribe, &top, shade::d2::BoxAlign::MiddleCenter, &self.title);
		}

		draw::DrawMenuItems {
			items_text: &wrap_items(&Self::ITEMS),
			selected_index: self.selected as usize,
		}.draw(&mut buf, &bottom, resx);

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
