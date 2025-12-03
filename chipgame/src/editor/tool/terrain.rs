use super::*;

fn put_tile(gs: &mut chipcore::GameState, cursor_pos: Vec2i, terrain: chipty::Terrain) {
	gs.set_terrain(cursor_pos, terrain);
	// Remove block entities if we are placing a dirt block terrain
	if matches!(terrain, chipty::Terrain::DirtBlock) {
		loop {
			let Some(ehandle) = gs.ents.iter().find_map(|ent| {
				if ent.pos == cursor_pos && matches!(ent.kind, chipty::EntityKind::Block) {
					return Some(ent.handle);
				}
				None
			}) else {break};
			gs.entity_remove(ehandle);
		}
	}
}

fn flood_fill(s: &mut EditorEditState, start: Vec2i, terrain: chipty::Terrain, offsets: &[Vec2i]) {
	let width = s.game.gs.field.width;
	let height = s.game.gs.field.height;
	if start.x < 0 || start.y < 0 || start.x >= width || start.y >= height {
		return;
	}

	let original = s.game.gs.field.get_terrain(start);
	if original == terrain {
		return;
	}

	let mut stack = Vec::new();
	put_tile(&mut s.game.gs, start, terrain);
	stack.push(start);

	while let Some(pos) = stack.pop() {
		for &offset in offsets {
			let neighbor = pos + offset;
			if neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= width || neighbor.y >= height {
				continue;
			}
			if s.game.gs.field.get_terrain(neighbor) != original {
				continue;
			}
			put_tile(&mut s.game.gs, neighbor, terrain);
			stack.push(neighbor);
		}
	}
}

static OFFSETS: [Vec2i; 4] = [
	Vec2i(1, 0),
	Vec2i(-1, 0),
	Vec2i(0, 1),
	Vec2i(0, -1),
];

fn put(s: &mut EditorEditState) {
	if s.input.key_shift {
		flood_fill(s, s.cursor_pos, s.selected_terrain, &OFFSETS);
	}
	else {
		put_tile(&mut s.game.gs, s.cursor_pos, s.selected_terrain);
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
