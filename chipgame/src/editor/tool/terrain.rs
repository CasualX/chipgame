use super::*;

fn put(s: &mut EditorEditState) {
	s.game.gs.set_terrain(s.cursor_pos, s.selected_terrain);
	// Remove block entities if we are placing a dirt block terrain
	if matches!(s.selected_terrain, chipty::Terrain::DirtBlock) {
		let cursor_pos = s.cursor_pos;
		loop {
			let Some(ehandle) = s.game.gs.ents.iter().find_map(|ent| {
				if ent.pos == cursor_pos && matches!(ent.kind, chipty::EntityKind::Block) {
					return Some(ent.handle);
				}
				None
			}) else {break};
			s.game.gs.entity_remove(ehandle);
		}
	}
	s.game.sync();
}

pub fn left_click(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		if s.cursor_pos.x < 0 || s.cursor_pos.y < 0 {
			s.sample();
		}
		s.tool_pos = Some(s.cursor_pos);
		put(s);
	}
}

pub fn right_click(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		s.sample();
	}
}

pub fn think(s: &mut EditorEditState) {
	if s.input.left_click {
		put(s);
	}
}
