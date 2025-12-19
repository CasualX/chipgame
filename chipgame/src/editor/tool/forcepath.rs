use super::*;

pub fn left_click(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		// Start a new force floor path with a random force tile at the cursor.
		s.fx.game.set_terrain(s.cursor_pos, Terrain::ForceRandom);
		s.forcepath_last_pos = Some(s.cursor_pos);
		s.tool_pos = Some(s.cursor_pos);
		s.fx.sync();
	}
	else {
		// Stop drawing.
		s.forcepath_last_pos = None;
		s.tool_pos = None;
	}

	s.input.left_click = pressed;
}

pub fn right_click(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		s.sample();
	}
}

pub fn think(s: &mut EditorEditState) {
	if !s.input.left_click {
		return;
	}
	let Some(last_pos) = s.forcepath_last_pos else {
		return;
	};
	let cursor_pos = s.cursor_pos;
	if cursor_pos == last_pos {
		return;
	}

	// Set the tiles to push in the direction we just moved
	let cursor_dir = chipty::Compass::from_vec(cursor_pos - last_pos);
	if let Some(cursor_dir) = cursor_dir {
		let terrain = force_tile_for_dir(cursor_dir);
		s.fx.game.set_terrain(last_pos, terrain);
		s.fx.game.set_terrain(cursor_pos, terrain);
	}

	s.forcepath_last_pos = Some(cursor_pos);
	s.fx.sync();
}

fn force_tile_for_dir(dir: chipty::Compass) -> chipty::Terrain {
	match dir {
		chipty::Compass::Up => chipty::Terrain::ForceN,
		chipty::Compass::Down => chipty::Terrain::ForceS,
		chipty::Compass::Left => chipty::Terrain::ForceW,
		chipty::Compass::Right => chipty::Terrain::ForceE,
	}
}
