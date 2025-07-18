use super::*;

const LEVELS_PER_PAGE: i32 = 14;

fn clip_offset(offset: i32, len: i32) -> i32 {
	i32::max(0, i32::min(len - LEVELS_PER_PAGE, offset))
}

#[derive(Default)]
pub struct LevelSelectMenu {
	pub selected: i32,
	pub offset: i32,
	pub items: Vec<(i32, String)>,
}

impl LevelSelectMenu {
	pub fn load_items(&mut self, lp: &crate::play::LevelSet, sd: &crate::play::SaveData) {
		self.items.clear();
		self.items.push((0, "Unlock level".to_string()));
		for &level_number in &sd.unlocked_levels {
			let Some(lv_info) = lp.lv_info.get((level_number - 1) as usize) else { continue };
			self.items.push((level_number, format!("Level {}: {}", level_number, lv_info.name)));
		}
	}

	fn jump(&mut self, jump: i32, events: &mut Vec<MenuEvent>) {
		let selected = i32::max(0, i32::min(self.items.len() as i32 - 1, self.selected + jump));
		if self.selected != selected {
			events.push(MenuEvent::CursorMove);
			self.selected = selected;
			if self.selected <= self.offset {
				self.offset = clip_offset(selected - 1, self.items.len() as i32);
			}
			else if self.selected >= self.offset + LEVELS_PER_PAGE - 1 {
				self.offset = clip_offset(selected - LEVELS_PER_PAGE + 2, self.items.len() as i32);
			}
		}
	}

	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			self.jump(-1, events);
		}
		if input.left.is_pressed() {
			self.jump(-10, events);
		}
		if input.down.is_pressed() {
			self.jump(1, events);
		}
		if input.right.is_pressed() {
			self.jump(10, events);
		}
		if input.a.is_pressed() || input.start.is_pressed() {
			let evt = match self.selected {
				0 => MenuEvent::UnlockLevel,
				index => MenuEvent::PlayLevel { level_number: self.items[index as usize].0 },
			};
			events.push(evt);
			events.push(MenuEvent::CursorSelect);
		}
		if input.b.is_pressed() {
			events.push(MenuEvent::CloseMenu);
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

		let size = resx.viewport.height() as f32 * FONT_SIZE;

		let scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: Vec4(255, 255, 255, 255),
			..Default::default()
		};

		let rect = Bounds2::point(Vec2(resx.viewport.width() as f32 * 0.5, size * 1.5));
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::TopCenter, "Go to level");

		// let mut pos = Vec2::ZERO;
		let mut scribe = shade::d2::Scribe {
			font_size: size * 0.75,
			line_height: size * 0.75 / 32.0 * 40.0,
			..Default::default()
		};

		let mut y = size * 5.0;
		for i in self.offset..i32::min(self.offset + LEVELS_PER_PAGE, self.items.len() as i32) {
			let item = &self.items[i as usize];
			let color = if i == self.selected { Vec4(255, 255, 255, 255) } else { Vec4(128, 128, 128, 255) };
			scribe.color = color;
			if self.offset != 0 && i == self.offset {
				scribe.color.w = 84;
			}
			if self.offset != self.items.len() as i32 - LEVELS_PER_PAGE && i == self.offset + LEVELS_PER_PAGE - 1 {
				scribe.color.w = 84;
			}

			let rect = Bounds2::point(Vec2(resx.viewport.width() as f32 * 0.25, y));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleLeft, &item.1);

			y += scribe.line_height;
		}
		// for (i, item) in self.items.iter().enumerate().filter(|&(i, _)| i >= self.offset as usize).take(7) {
		// 	let color = if i == self.selected as usize { Vec4(255, 255, 255, 255) } else { Vec4(128, 128, 128, 255) };
		// 	scribe.color = color;

		// 	let rect = Bounds2::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 * 0.5 - 100.0 + i as i32 as f32 * scribe.line_height));
		// 	buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleLeft, item);
		// }

		buf.draw(g, shade::Surface::BACK_BUFFER);
	}
}
