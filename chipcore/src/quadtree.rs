use super::*;

/// Entity lookup acceleration structure.
///
/// Not really a QuadTree but it serves the same purpose ;)  
/// This is a simple 2D grid of entity handles to quickly find entities at a given position.
#[derive(Default)]
pub struct QuadTree {
	width: i32,
	height: i32,
	tiles: Vec<[EntityHandle; 4]>,
}

impl QuadTree {
	pub fn init(&mut self, width: i32, height: i32) {
		self.width = width;
		self.height = height;
		let len = (width * height) as usize;
		self.tiles.clear();
		self.tiles.resize(len, [EntityHandle::INVALID; 4]);
	}

	pub fn add(&mut self, ehandle: EntityHandle, pos: Vec2i) {
		let index = (pos.y * self.width + pos.x) as usize;
		if let Some(tile) = self.tiles.get_mut(index) {
			if tile[0] == EntityHandle::INVALID {
				tile[0] = ehandle;
			}
			else if tile[1] == EntityHandle::INVALID {
				tile[1] = ehandle;
			}
			else if tile[2] == EntityHandle::INVALID {
				tile[2] = ehandle;
			}
			else if tile[3] == EntityHandle::INVALID {
				tile[3] = ehandle;
			}
			else {
				#[cfg(debug_assertions)]
				panic!("QuadTree tile at {} is full: {:?}!", pos, tile);
			}
		}
	}

	pub fn remove(&mut self, ehandle: EntityHandle, pos: Vec2i) {
		let index = (pos.y * self.width + pos.x) as usize;
		if let Some(tile) = self.tiles.get_mut(index) {
			if tile[0] == ehandle {
				tile[0] = EntityHandle::INVALID;
			}
			if tile[1] == ehandle {
				tile[1] = EntityHandle::INVALID;
			}
			if tile[2] == ehandle {
				tile[2] = EntityHandle::INVALID;
			}
			if tile[3] == ehandle {
				tile[3] = EntityHandle::INVALID;
			}
		}
	}

	pub fn update(&mut self, ehandle: EntityHandle, old_pos: Vec2i, new_pos: Vec2i) {
		self.remove(ehandle, old_pos);
		self.add(ehandle, new_pos);
	}

	pub fn get(&self, pos: Vec2i) -> [EntityHandle; 4] {
		let index = (pos.y * self.width + pos.x) as usize;
		*self.tiles.get(index).unwrap_or(&[EntityHandle::INVALID; 4])
	}
}
