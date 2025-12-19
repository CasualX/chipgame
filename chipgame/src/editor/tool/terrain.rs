use super::*;

#[derive(Clone)]
pub struct TerrainToolState {
	pub selected_terrain: chipty::Terrain,
}

impl Default for TerrainToolState {
	fn default() -> TerrainToolState {
		TerrainToolState {
			selected_terrain: chipty::Terrain::Floor,
		}
	}
}

impl TerrainToolState {
	pub fn left_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		if pressed {
			self.put(s);
		}
	}

	pub fn right_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		if pressed {
			s.sample();
		}
	}

	pub fn think(&mut self, s: &mut EditorEditState) {
		if s.input.left_click {
			self.put(s);
		}
	}

	fn put(&mut self, s: &mut EditorEditState) {
		if s.input.key_shift {
			flood_fill(s, s.cursor_pos, self.selected_terrain);
		}
		else {
			put_tile(&mut s.fx.game, s.cursor_pos, self.selected_terrain);
		}
		s.fx.sync();
	}
}

fn put_tile(game: &mut chipcore::GameState, cursor_pos: Vec2i, terrain: chipty::Terrain) {
	game.set_terrain(cursor_pos, terrain);
	// Remove block entities if we are placing a dirt block terrain
	if matches!(terrain, chipty::Terrain::DirtBlock) {
		loop {
			let Some(ehandle) = game.ents.iter().find_map(|ent| {
				if ent.pos == cursor_pos && matches!(ent.kind, chipty::EntityKind::Block) {
					return Some(ent.handle);
				}
				None
			}) else {break};
			game.entity_remove(ehandle);
		}
	}
}

static OFFSETS: [Vec2i; 4] = [
	Vec2i(1, 0),
	Vec2i(-1, 0),
	Vec2i(0, 1),
	Vec2i(0, -1),
];

fn flood_fill(s: &mut EditorEditState, start: Vec2i, terrain: chipty::Terrain) {
	let width = s.fx.game.field.width;
	let height = s.fx.game.field.height;
	if start.x < 0 || start.y < 0 || start.x >= width || start.y >= height {
		return;
	}

	let original = s.fx.game.field.get_terrain(start);
	if original == terrain {
		return;
	}

	let mut stack = Vec::new();
	put_tile(&mut s.fx.game, start, terrain);
	stack.push(start);

	while let Some(pos) = stack.pop() {
		for &offset in &OFFSETS {
			let neighbor = pos + offset;
			if neighbor.x < 0 || neighbor.y < 0 || neighbor.x >= width || neighbor.y >= height {
				continue;
			}
			if s.fx.game.field.get_terrain(neighbor) != original {
				continue;
			}
			put_tile(&mut s.fx.game, neighbor, terrain);
			stack.push(neighbor);
		}
	}
}
