use super::*;

#[derive(Default)]
pub struct GameOverMenu {
	pub selected: u8,
	pub activity: chipcore::PlayerActivity,
	pub level_number: i32,
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
				0 => MenuEvent::RetryLevel,
				_ => MenuEvent::OpenMainMenu,
			};
			events.push(evt);
			events.push(MenuEvent::CursorSelect);
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

		let [top, middle, bottom] = draw::flexv(rect, None, layout::Justify::Start, &[layout::Unit::Fr(1.0); 3]);

		let reason = {
			use chipcore::PlayerActivity as PA;
			match self.activity {
				PA::Drowned => "Ooops! Chip can't swim without flippers!",
				PA::Burned => "Ooops! Don't step in the fire without fire boots!",
				PA::Bombed => "Ooops! Don't touch the bombs!",
				PA::OutOfTime => "Ooops! Out of time!",
				PA::Collided => "Ooops! Watch out for moving blocks!",
				PA::Eaten => "Ooops! Look out for creatures!",
				PA::NotOkay => "Ooops! You're not okay!",
				_ => "Game Over!",
			}
		};

		draw::DrawPlayTitle {
			level_number: self.level_number,
			level_name: &self.level_name,
			subtitle: Some(&fmtools::fmt!("\x1b[color=#f08]"{reason})),
		}.draw(&mut buf, &top, resx);

		let [_, middle, _] = draw::flexh(middle, None, layout::Justify::Center, &[layout::Unit::Fr(1.0); 3]);

		draw::DrawScoreCard {
			attempts: self.attempts,
			time: self.time,
			steps: self.steps,
			bonks: self.bonks,
			time_high_score: -2,
			steps_high_score: -2,
		}.draw(&mut buf, &middle, resx);

		draw::DrawMenuItems {
			items_text: &wrap_items(&Self::ITEMS),
			selected_index: self.selected as usize,
		}.draw(&mut buf, &bottom, resx);

		buf.draw(g, shade::Surface::BACK_BUFFER);
	}
}
