use super::*;

#[derive(Default)]
pub struct PauseMenu {
	pub selected: u8,
	pub level_number: i32,
	pub level_name: String,
}

impl PauseMenu {
	const ITEMS: [&'static str; 6] = ["Resume", "Set Warp Point", "Warp Back", "Restart", "Options", "Main Menu"];
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
		if input.a.is_pressed() {
			let evt = match self.selected {
				0 => MenuEvent::ResumePlay,
				1 => MenuEvent::SaveState,
				2 => MenuEvent::LoadState,
				3 => MenuEvent::RestartLevel,
				4 => MenuEvent::OpenOptions,
				_ => MenuEvent::OpenMainMenu,
			};
			events.push(evt);
			events.push(MenuEvent::CursorSelect);
		}
		if input.b.is_pressed() || input.start.is_pressed() {
			events.push(MenuEvent::ResumePlay);
		}
		if input.select.is_pressed() {
			events.push(MenuEvent::CloseMenu);
			events.push(MenuEvent::OpenScoutMode);
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

		let [top, bottom] = draw::flexv(rect, None, layout::Justify::Start, &[layout::Unit::Fr(1.0), layout::Unit::Fr(2.0)]);

		draw::DrawPlayTitle {
			level_number: self.level_number,
			level_name: &self.level_name,
			subtitle: Some(&"\x1b[color=#0f0]Paused!"),
		}.draw(&mut buf, &top, resx);

		draw::DrawMenuItems {
			items_text: &wrap_items(&Self::ITEMS),
			selected_index: self.selected as usize,
		}.draw(&mut buf, &bottom, resx);

		buf.draw(g, shade::Surface::BACK_BUFFER);
	}
}
