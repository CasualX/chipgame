use super::*;

pub fn left_click(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		s.conn_src = s.cursor_pos;

		if s.cursor_pos.x < 0 || s.cursor_pos.y < 0 {
			s.sample();
		}
	}
	else {
		let new_conn = FieldConn { src: s.conn_src, dest: s.cursor_pos };

		if new_conn.src != new_conn.dest {
			if let Some(index) = s.fx.game.field.conns.iter().position(|conn| conn == &new_conn) {
				s.fx.game.field.conns.remove(index);
			}
			else {
				s.fx.game.field.conns.push(new_conn);
			}
		}
	}
}
pub fn think(_s: &mut EditorEditState) { }
pub fn right_click(_s: &mut EditorEditState, _pressed: bool) { }
