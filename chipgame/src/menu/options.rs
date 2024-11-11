use super::*;

#[derive(Default)]
pub struct OptionsMenu {
	pub selected: u8,
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
}

impl OptionsMenu {
	const ITEMS: [&'static str; 4] = ["Background music: ", "Sound effects: ", "Developer mode: ", "Back"];
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
				0 => {
					self.bg_music = !self.bg_music;
					if self.bg_music { MenuEvent::BgMusicOn } else { MenuEvent::BgMusicOff }
				}
				1 => {
					self.sound_fx = !self.sound_fx;
					if self.sound_fx { MenuEvent::SoundFxOn } else { MenuEvent::SoundFxOff }
				}
				2 => {
					self.dev_mode = !self.dev_mode;
					if self.dev_mode { MenuEvent::DevModeOn } else { MenuEvent::DevModeOff }
				}
				_ => MenuEvent::CloseMenu,
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

		// let mut pos = Vec2::ZERO;
		let mut scribe = shade::d2::Scribe {
			font_size: 32.0,
			line_height: 40.0,
			..Default::default()
		};

		for (i, item) in Self::ITEMS.iter().enumerate() {
			let color = if i == self.selected as usize { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;

			let state = match i {
				0 => Some(self.bg_music),
				1 => Some(self.sound_fx),
				2 => Some(self.dev_mode),
				_ => None,
			};

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 * 0.5 - 100.0 + i as i32 as f32 * scribe.line_height));
			if let Some(state) = state {
				let color = if state { "\x1b[color=#0f0]ON" } else { "\x1b[color=#f00]OFF" };
				buf.text_fmt_lines(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, &[format_args!("{}{}", item, color)]);
			}
			else {
				buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, item);
			}
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
