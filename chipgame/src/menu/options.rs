use super::*;

#[derive(Default)]
pub struct OptionsMenu {
	pub selected: u8,
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
	pub perspective: bool,
}

impl OptionsMenu {
	const ITEMS: [&'static str; 4] = ["Background music: ", "Sound effects: ", "Perspective: ", "Back"];
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
					MenuEvent::SetBackgroundMusic { value: self.bg_music }
				}
				1 => {
					self.sound_fx = !self.sound_fx;
					MenuEvent::SetSoundEffects { value: self.sound_fx }
				}
				2 => {
					self.perspective = !self.perspective;
					MenuEvent::SetPerspective { value: self.perspective }
				}
				_ => MenuEvent::CloseMenu,
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

		let get_flag = |state| if state { "\x1b[color=#0f0]ON" } else { "\x1b[color=#f00]OFF" };

		let items: [&dyn fmt::Display; 4] = [
			&fmtools::fmt!("Background music: "{get_flag(self.bg_music)}),
			&fmtools::fmt!("Sound effects: "{get_flag(self.sound_fx)}),
			&fmtools::fmt!("Perspective: "{get_flag(self.perspective)}),
			&"Back",
		];

		draw::DrawMenuItems {
			items_text: &items,
			selected_index: self.selected as usize,
		}.draw(&mut buf, &rect, resx);

		buf.draw(g, shade::Surface::BACK_BUFFER);
	}
}
