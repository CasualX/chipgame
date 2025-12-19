use super::*;

pub fn left_click(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		// Start a new ice path
		s.fx.game.set_terrain(s.cursor_pos, chipty::Terrain::Ice);
		s.icepath_last_pos = Some(s.cursor_pos);
		s.icepath_last_dir = None;
		s.tool_pos = Some(s.cursor_pos);
		s.fx.sync();
	}
	else {
		// Stop drawing
		s.icepath_last_pos = None;
		s.icepath_last_dir = None;
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
	let Some(last_pos) = s.icepath_last_pos else {
		return;
	};
	let cursor_pos = s.cursor_pos;
	if cursor_pos == last_pos {
		return;
	}

	// Set the previous tile to the appropriate ice corner
	let cursor_dir = chipty::Compass::from_vec(cursor_pos - last_pos);
	if let (Some(last_dir), Some(cursor_dir)) = (s.icepath_last_dir, cursor_dir) {
		let terrain = ice_tile_for_turn(last_dir, cursor_dir);
		s.fx.game.set_terrain(last_pos, terrain);
	}

	// Mark the current tile as ice terrain
	s.fx.game.set_terrain(cursor_pos, chipty::Terrain::Ice);
	s.icepath_last_pos = Some(cursor_pos);
	s.icepath_last_dir = cursor_dir;
	s.fx.sync();
}

fn ice_tile_for_turn(from: chipty::Compass, to: chipty::Compass) -> chipty::Terrain {
	match (from, to) {
		(chipty::Compass::Up, chipty::Compass::Right) => chipty::Terrain::IceNW,
		(chipty::Compass::Up, chipty::Compass::Left) => chipty::Terrain::IceNE,
		(chipty::Compass::Right, chipty::Compass::Down) => chipty::Terrain::IceNE,
		(chipty::Compass::Right, chipty::Compass::Up) => chipty::Terrain::IceSE,
		(chipty::Compass::Down, chipty::Compass::Left) => chipty::Terrain::IceSE,
		(chipty::Compass::Down, chipty::Compass::Right) => chipty::Terrain::IceSW,
		(chipty::Compass::Left, chipty::Compass::Down) => chipty::Terrain::IceNW,
		(chipty::Compass::Left, chipty::Compass::Up) => chipty::Terrain::IceSW,
		_ => chipty::Terrain::Ice,
	}
}
