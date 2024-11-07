use super::*;

pub fn left_click(s: &mut EditorState, pressed: bool) {
	if pressed {
		s.conn_src = s.cursor_pos;

		if s.cursor_pos.x < 0 || s.cursor_pos.y < 0 {
			s.sample();
		}
	}
	else {
		let new_conn = core::Conn { src: s.conn_src, dest: s.cursor_pos };

		if new_conn.src != new_conn.dest {
			if let Some(index) = s.game.gs.field.conns.iter().position(|conn| conn == &new_conn) {
				s.game.gs.field.conns.remove(index);
			}
			else {
				s.game.gs.field.conns.push(new_conn);
			}
		}
	}
}
pub fn think(_s: &mut EditorState) { }
pub fn right_click(_s: &mut EditorState, _pressed: bool) { }
