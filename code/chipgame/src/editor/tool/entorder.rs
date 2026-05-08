use super::*;

#[derive(Clone, Default)]
pub struct EntOrderToolState {
}

impl EntOrderToolState {
	pub fn left_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		if pressed {
			self.move_entity(s, true);
		}
	}

	pub fn right_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		if pressed {
			self.move_entity(s, false);
		}
	}

	pub fn think(&mut self, _s: &mut EditorEditState) {
	}

	fn move_entity(&self, s: &mut EditorEditState, inc: bool) {
		let cursor_pos = s.cursor_pos;
		let mut level = s.save_level_dto();
		if let Some(index) = level.entities.iter().position(|ent| ent.pos == cursor_pos) {
			let new_index = if inc { usize::min(index + 1, level.entities.len() - 1) } else { index.saturating_sub(1) };
			if index != new_index {
				level.entities.swap(index, new_index);
				// Reload the level
				let json = serde_json::to_string(&level).unwrap();
				s.reload_level(&json);
			}
		}
	}
}
