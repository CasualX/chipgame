use super::*;

/// Game playfield.
#[derive(Clone, Default)]
pub struct Field {
	pub name: String,
	pub author: Option<String>,
	pub hint: Option<String>,
	pub password: Option<String>,
	pub seed: u64,
	pub time_limit: i32,
	pub required_chips: i32,
	pub width: i32,
	pub height: i32,
	pub terrain: Vec<Terrain>,
	pub conns: Vec<FieldConn>,
	pub camera_triggers: Vec<chipty::CameraFocusTrigger>,
	pub replays: Option<Vec<chipty::ReplayDto>>,
	pub trophies: Option<chipty::Trophies>,
}

impl Field {
	pub(super) fn parse(&mut self, ld: &chipty::LevelDto, seed: u64) {
		self.name.clone_from(&ld.name);
		self.author.clone_from(&ld.author);
		self.hint.clone_from(&ld.hint);
		self.password.clone_from(&ld.password);
		self.seed = seed;
		self.time_limit = ld.time_limit;
		self.required_chips = ld.required_chips;
		self.width = ld.map.width;
		self.height = ld.map.height;
		self.terrain.clear();
		self.conns.clone_from(&ld.connections);
		self.camera_triggers.clone_from(&ld.camera_triggers);
		self.replays.clone_from(&ld.replays);
		self.trophies.clone_from(&ld.trophies);

		assert!(
			ld.map.width >= chipty::FIELD_MIN_WIDTH && ld.map.width <= FIELD_MAX_WIDTH &&
			ld.map.height >= chipty::FIELD_MIN_HEIGHT && ld.map.height <= FIELD_MAX_HEIGHT,
			"Invalid map size width={} height={}", ld.map.width, ld.map.height);
		let size = ld.map.width as usize * ld.map.height as usize;
		self.terrain.reserve_exact(size);

		if ld.map.data.is_empty() {
			self.terrain.resize(size, Terrain::Floor);
		}
		else {
			assert_eq!(ld.map.data.len(), size, "Invalid map data length");
			for y in 0..ld.map.height {
				for x in 0..ld.map.width {
					let index = (y * ld.map.width + x) as usize;
					let terrain = ld.map.legend[ld.map.data[index] as usize];
					self.terrain.push(terrain);
				}
			}
		}
	}
	pub fn get_terrain(&self, pos: Vec2i) -> Terrain {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return Terrain::Wall;
		}
		let index = (y * self.width + x) as usize;
		self.terrain.get(index).cloned().unwrap_or(Terrain::Wall)
	}
	pub(super) fn set_terrain(&mut self, pos: Vec2i, terrain: Terrain) -> Option<Terrain> {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return None;
		}
		let index = (y * self.width + x) as usize;
		let ptr = self.terrain.get_mut(index)?;
		let old = *ptr;
		if old == terrain {
			return None;
		}
		*ptr = terrain;
		Some(old)
	}
	pub fn find_teleport_dest(&self, src: Vec2i) -> Option<Vec2i> {
		// Use connections to find teleport destination
		if let Some(conn) = self.conns.iter().find(|conn| conn.src == src) {
			return Some(conn.dest);
		}
		// If no explicit connection, find another teleport in reverse reading order
		let mut pos = src;
		loop {
			pos.x -= 1;
			if pos.x < 0 {
				pos.x = self.width - 1;
				pos.y -= 1;
				if pos.y < 0 {
					pos.y = self.height - 1;
				}
			}
			if pos == src {
				return None;
			}
			if matches!(self.get_terrain(pos), Terrain::Teleport) {
				return Some(pos);
			}
		}
	}
	pub fn find_conn_by_src(&self, pos: Vec2i) -> Option<&FieldConn> {
		self.conns.iter().find(|conn| conn.src == pos)
	}
	pub fn is_pos_inside(&self, pos: Vec2i) -> bool {
		pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
	}
}
