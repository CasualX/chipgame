use super::*;

const LEVELS_PER_PAGE: i32 = 14;

fn clip_offset(offset: i32, len: i32) -> i32 {
	i32::min(len - LEVELS_PER_PAGE, i32::max(0, offset))
}

#[derive(Default)]
pub struct LevelSelectMenu {
	pub selected: i32,
	pub offset: i32,
	pub items: Vec<String>,
}

impl LevelSelectMenu {
	pub fn load_items(&mut self, lp: &crate::play::LevelPack) {
		self.items.clear();
		self.items.push("Unlock level".to_string());
		for (index, ld) in lp.lv_info.iter().enumerate() {
			self.items.push(format!("Level {}: {}", index + 1, ld.name));
		}
	}
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			if self.selected > 0 {
				events.push(MenuEvent::CursorMove);
				self.selected = self.selected - 1;
			}
			if self.selected < self.offset + 1 {
				self.offset = clip_offset(self.selected - 1, self.items.len() as i32);
			}
		}
		if input.down.is_pressed() {
			if self.selected < self.items.len() as i32 - 1 {
				events.push(MenuEvent::CursorMove);
				self.selected = self.selected + 1;
			}
			if self.selected >= self.offset + (LEVELS_PER_PAGE - 1) {
				self.offset = clip_offset(self.selected - (LEVELS_PER_PAGE - 1) + 1, self.items.len() as i32);//i32::min(self.items.len() as i32 - LEVELS_PER_PAGE, i32::max(0, self.selected - (LEVELS_PER_PAGE - 1) + 1));
			}
		}
		if input.a.is_pressed() || input.start.is_pressed() {
			let evt = match self.selected {
				0 => MenuEvent::UnlockLevel,
				index => MenuEvent::PlayLevel { level_number: index },
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

		let scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, size * 1.5));
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, "Go to level");

		// let mut pos = Vec2::ZERO;
		let mut scribe = shade::d2::Scribe {
			font_size: size * 0.75,
			line_height: size * 0.75 / 32.0 * 40.0,
			..Default::default()
		};

		let mut y = size * 5.0;
		for i in self.offset..i32::min(self.offset + LEVELS_PER_PAGE, self.items.len() as i32) {
			let item = &self.items[i as usize];
			let color = if i == self.selected { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;
			if self.offset != 0 && i == self.offset {
				scribe.color.w = 84;
			}
			if self.offset != self.items.len() as i32 - LEVELS_PER_PAGE && i == self.offset + LEVELS_PER_PAGE - 1 {
				scribe.color.w = 84;
			}

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.25, y));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleLeft, item);

			y += scribe.line_height;
		}
		// for (i, item) in self.items.iter().enumerate().filter(|&(i, _)| i >= self.offset as usize).take(7) {
		// 	let color = if i == self.selected as usize { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
		// 	scribe.color = color;

		// 	let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 * 0.5 - 100.0 + i as i32 as f32 * scribe.line_height));
		// 	buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleLeft, item);
		// }

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
