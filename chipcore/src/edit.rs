use super::*;

impl GameState {
	/// Swaps all keys and locks of one color to another.
	pub fn swap_keys(&mut self, old: KeyColor, new: KeyColor) {
		let old_terrain = match old {
			KeyColor::Blue => Terrain::BlueLock,
			KeyColor::Red => Terrain::RedLock,
			KeyColor::Green => Terrain::GreenLock,
			KeyColor::Yellow => Terrain::YellowLock,
		};
		let new_terrain = match new {
			KeyColor::Blue => Terrain::BlueLock,
			KeyColor::Red => Terrain::RedLock,
			KeyColor::Green => Terrain::GreenLock,
			KeyColor::Yellow => Terrain::YellowLock,
		};
		for y in 0..self.field.height {
			for x in 0..self.field.width {
				let pos = Vec2i::new(x, y);
				let terrain = self.field.get_terrain(pos);
				if terrain == old_terrain {
					self.set_terrain(pos, new_terrain);
				}
				else if terrain == new_terrain {
					self.set_terrain(pos, old_terrain);
				}
			}
		}

		let old_kind = match old {
			KeyColor::Blue => EntityKind::BlueKey,
			KeyColor::Red => EntityKind::RedKey,
			KeyColor::Green => EntityKind::GreenKey,
			KeyColor::Yellow => EntityKind::YellowKey,
		};
		let new_kind = match new {
			KeyColor::Blue => EntityKind::BlueKey,
			KeyColor::Red => EntityKind::RedKey,
			KeyColor::Green => EntityKind::GreenKey,
			KeyColor::Yellow => EntityKind::YellowKey,
		};
		for ent in self.ents.iter_mut() {
			// FIXME: Notify about entity kind change
			if ent.kind == old_kind {
				ent.kind = new_kind;
			}
		}
	}

	/// Applies a level brush at the given position.
	pub fn brush_apply(&mut self, pos: Vec2i, brush: &chipty::LevelBrush) {
		assert!(brush.width > 0 && brush.height > 0, "Invalid brush size");
		assert_eq!(brush.terrain.len(), (brush.width * brush.height) as usize, "Invalid brush terrain data length");

		// Write terrain
		for by in 0..brush.height {
			for bx in 0..brush.width {
				if let Some(terrain) = brush.terrain[(by * brush.width + bx) as usize] {
					let dest = pos + Vec2i::new(bx, by);
					self.set_terrain(dest, terrain);
				}
			}
		}

		// Add entities
		for &ent_args in &brush.entities {
			let pos = pos + ent_args.pos;
			let args = chipty::EntityArgs { pos, ..ent_args };
			self.entity_create(&args);
		}

		// Add connections
		for conn in &brush.connections {
			// Only offset when the connection is inside the brush area
			let src = if brush.is_pos_inside(conn.src) { pos + conn.src } else { conn.src };
			let dest = if brush.is_pos_inside(conn.dest) { pos + conn.dest } else { conn.dest };
			// Skip connections outside the field
			if self.field.is_pos_inside(src) && self.field.is_pos_inside(dest) {
				self.field.conns.push(chipty::FieldConn { src, dest });
			}
		}
	}

	/// Creates a level brush from the current level.
	pub fn brush_create(&self) -> chipty::LevelBrush {
		let terrain = self.field.terrain.iter().cloned().map(Some).collect();
		let entities = self.ents.iter().map(|ent| ent.to_entity_args()).collect();
		let connections = self.field.conns.clone();
		chipty::LevelBrush {
			width: self.field.width,
			height: self.field.height,
			terrain,
			entities,
			connections,
		}
	}
}
