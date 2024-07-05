use super::*;

/// Field map data transfer object.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MapDto {
	pub width: i32,
	pub height: i32,
	pub data: Vec<u8>,
	pub legend: Vec<Terrain>,
}

/// Field data transfer object.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldDto {
	pub name: String,
	pub hint: String,
	pub password: String,
	pub seed: u64,
	pub time: i32,
	pub chips: i32,
	pub map: MapDto,
	pub entities: Vec<EntityArgs>,
	pub connections: Vec<Conn>,
}

/// Connection between terrain tiles.
///
/// The connection defines the relationship between:
/// * A red button and associated clone machine terrain.
/// * A brown button and associated bear trap terrain.
/// * A teleport pad and destination terrain.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Conn {
	pub src: Vec2i,
	pub dest: Vec2i,
}

/// Game playfield.
#[derive(Default)]
pub struct Field {
	pub name: String,
	pub hint: String,
	pub password: String,
	pub seed: u64,
	pub time: i32,
	pub chips: i32,
	pub width: i32,
	pub height: i32,
	pub terrain: Vec<Terrain>,
	pub conns: Vec<Conn>,
}

impl Field {
	pub fn get_terrain(&self, pos: Vec2i) -> Terrain {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return Terrain::Blank;
		}
		let index = (y * self.width + x) as usize;
		self.terrain.get(index).cloned().unwrap_or(Terrain::Blank)
	}
	pub fn set_terrain(&mut self, pos: Vec2i, terrain: Terrain) {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return;
		}
		let index = (y * self.width + x) as usize;
		if let Some(ptr) = self.terrain.get_mut(index) {
			*ptr = terrain;
		}
	}
	pub fn find_conn_by_src(&self, pos: Vec2i) -> Option<&Conn> {
		self.conns.iter().find(|conn| conn.src == pos)
	}
	pub fn find_conn_by_dest(&self, pos: Vec2i) -> Option<&Conn> {
		self.conns.iter().find(|conn| conn.dest == pos)
	}
}

impl Field {
	pub fn can_move(&self, pos: Vec2i, dir: Compass, flags: &SolidFlags) -> bool {
		// Check for panels on the current terrain
		let cur_terrain = self.get_terrain(pos);
		let cur_solid = cur_terrain.solid_flags();
		let panel = match dir {
			Compass::Up => PANEL_N,
			Compass::Left => PANEL_W,
			Compass::Down => PANEL_S,
			Compass::Right => PANEL_E,
		};
		// If on a solid wall, allow movement out
		if cur_solid != SOLID_WALL && cur_solid & panel != 0 {
			return false;
		}

		let next_terrain = self.get_terrain(pos + dir.to_vec());
		let next_solid = next_terrain.solid_flags();

		// Check the solid flags of the next terrain
		let panel = match dir {
			Compass::Up => PANEL_S,
			Compass::Left => PANEL_E,
			Compass::Down => PANEL_N,
			Compass::Right => PANEL_W,
		};
		if next_solid & panel != 0 {
			return false;
		}

		if flags.gravel && matches!(next_terrain, Terrain::Gravel) {
			return false;
		}
		if flags.fire && matches!(next_terrain, Terrain::Fire) {
			return false;
		}
		if flags.dirt && matches!(next_terrain, Terrain::Dirt) {
			return false;
		}
		if flags.exit && matches!(next_terrain, Terrain::Exit) {
			return false;
		}

		return true;
	}
}
