use super::*;

const LEVELS_PER_PAGE: i32 = 14;

fn clip_offset(offset: i32, len: i32) -> i32 {
	i32::max(0, i32::min(len - LEVELS_PER_PAGE, offset))
}

#[derive(Default)]
pub struct LevelSelectMenu {
	pub selected: i32,
	pub offset: i32,
	pub offsetf: f32,
	pub items: Vec<(i32, String)>,
	pub preview: Option<Box<crate::fx::FxState>>,
}

impl LevelSelectMenu {
	pub fn load_items(&mut self, lp: &crate::play::LevelSet, sd: &crate::play::SaveData) {
		self.items.clear();
		self.items.push((0, "Unlock level".to_string()));
		for &level_number in &sd.unlocked_levels {
			let Some(lv_info) = lp.lv_info.get((level_number - 1) as usize) else { continue };
			if sd.current_level == level_number {
				self.selected = self.items.len() as i32;
			}
			self.items.push((level_number, format!("Level {}: {}", level_number, lv_info.name)));
		}
		self.offset = clip_offset(self.selected - LEVELS_PER_PAGE / 2 + 1, self.items.len() as i32);
		self.offsetf = self.offset as f32;
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

			// Request the level preview
			let level_number = self.items[selected as usize].0;
			events.push(MenuEvent::LevelPreview { level_number });
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
		// Draw the level preview when available
		if let Some(fx) = &mut self.preview {
			fx.draw(g, resx);
			darken(g, resx, 168);
		}

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

		let rect = Bounds2::point(Vec2(resx.viewport.width() as f32 * 0.5, size * 1.5), Vec2::ZERO);
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::TopCenter, "Go to level");

		let mut scribe = shade::d2::Scribe {
			font_size: size * 0.75,
			line_height: size * 0.75 / 32.0 * 40.0,
			..Default::default()
		};

		self.offsetf = Vec2(self.offsetf, 0.0).exp_decay(Vec2(self.offset as f32, 0.0), 15.0, 1.0 / 60.0).x;

		let mut y = size * 5.0 - self.offsetf * scribe.line_height;
		let top_transparent = size * 5.0 - scribe.line_height * 1.5;
		let top_opaque = size * 5.0;
		let bottom_opaque = top_opaque + (LEVELS_PER_PAGE - 1) as f32 * scribe.line_height;
		let bottom_transparent = bottom_opaque + scribe.line_height * 1.5;

		for i in 0..self.items.len() as i32 {
			if y < top_transparent {
				y += scribe.line_height;
				continue;
			}
			else if y > bottom_transparent {
				break;
			}

			let alpha = if y < top_opaque {
				(y - top_transparent) / (top_opaque - top_transparent)
			}
			else if y > bottom_opaque {
				(bottom_transparent - y) / (bottom_transparent - bottom_opaque)
			}
			else {
				1.0
			};
			let alpha = (f32::clamp(alpha, 0.0, 1.0) * 255.0) as u8;
			let color = if i == self.selected { 255 } else { 128 };
			scribe.color = Vec4(color, color, color, alpha);
			scribe.outline.w = alpha;

			let item = &self.items[i as usize];
			let rect = Bounds2::point(Vec2(resx.viewport.width() as f32 * 0.25, y), Vec2::ZERO);
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleLeft, &item.1);

			y += scribe.line_height;
		}

		buf.draw(g, shade::Surface::BACK_BUFFER);
	}
}
