use super::*;

pub fn left_click(s: &mut EditorState, pressed: bool) {
	if pressed {
		if s.cursor_pos.x < 0 || s.cursor_pos.y < 0 {
			s.sample();
		}
		s.tool_pos = Some(s.cursor_pos);
		s.game.gs.set_terrain(s.cursor_pos, s.selected_terrain);
		s.game.sync();
	}
}

pub fn right_click(s: &mut EditorState, pressed: bool) {
	if pressed {
		s.sample();
	}
}

pub fn think(s: &mut EditorState) {
	if s.input.left_click {
		s.game.gs.set_terrain(s.cursor_pos, s.selected_terrain);
		s.game.sync();
	}
}
