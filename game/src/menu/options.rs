use super::*;

#[derive(Default)]
pub struct OptionsMenu {
	pub selected: u8,
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
	pub back_menu: Option<MenuEvent>,
}

impl OptionsMenu {
	const ITEMS: [&'static str; 4] = ["Background music: ", "Sound effects: ", "Developer mode: ", "Back"];
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			self.selected = if self.selected > 0 { self.selected - 1 } else { self.selected };
		}
		if input.down.is_pressed() {
			self.selected = if self.selected < Self::ITEMS.len() as u8 - 1 { self.selected + 1 } else { self.selected };
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
				_ => self.back_menu.clone().unwrap_or(MenuEvent::MainMenu),
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

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 * 0.5 - 100.0 + i as i32 as f32 * scribe.line_height));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, item);
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
