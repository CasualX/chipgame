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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub author: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hint: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub password: Option<String>,
	pub required_chips: i32,
	pub time_limit: i32,
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
	pub author: Option<String>,
	pub hint: Option<String>,
	pub password: Option<String>,
	pub seed: u64,
	pub time_limit: i32,
	pub required_chips: i32,
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
	pub(super) fn set_terrain(&mut self, pos: Vec2i, terrain: Terrain) -> Option<Terrain> {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return None;
		}
		let index = (y * self.width + x) as usize;
		let ptr = self.terrain.get_mut(index)?;
		let old = *ptr;
		*ptr = terrain;
		Some(old)
	}
	pub fn find_conn_by_src(&self, pos: Vec2i) -> Option<&Conn> {
		self.conns.iter().find(|conn| conn.src == pos)
	}
}

impl Field{
	pub(crate) fn resize_bottom(&mut self, additional: i32, terrain: Terrain) {
		let new_height = clamp_field_size(self.height + additional);
		self.terrain.resize((self.width * new_height) as usize, terrain);
		self.height = new_height;
	}
	pub(crate) fn resize_top(&mut self, additional: i32, terrain: Terrain) {
		let new_height = clamp_field_size(self.height + additional);
		if additional > 0 {
			self.terrain.resize((self.width * new_height) as usize, terrain);
			// self.terrain.rotate_right((self.width * additional) as usize);
			self.terrain.copy_within(0..(self.width * self.height) as usize, (additional * self.width) as usize);
			self.terrain[..(additional * self.width) as usize].fill(terrain);
		}
		else {
			self.terrain.drain(0..(-additional * self.width) as usize);
		}
		self.height = new_height;
	}
	pub(crate) fn resize_right(&mut self, additional: i32, terrain: Terrain) {
		let new_width = clamp_field_size(self.width + additional);
		if additional > 0 {
			self.terrain.resize((new_width * self.height) as usize, terrain);
			for y in (0..self.height).rev() {
				let src = y * self.width;
				let dest = y * new_width;
				self.terrain.copy_within(src as usize..(src + self.width) as usize, dest as usize);
				self.terrain[dest as usize..(dest + additional) as usize].fill(terrain);
			}
		}
		else {
			for y in 1..self.height {
				let src = y * self.width;
				let dest = y * new_width;
				self.terrain.copy_within(src as usize..(src + self.width) as usize, dest as usize);
			}
			self.terrain.truncate((new_width * self.height) as usize);
		}
		self.width = new_width;
	}
	pub(crate) fn resize_left(&mut self, additional: i32, terrain: Terrain) {
		let new_width = clamp_field_size(self.width + additional);
		if additional > 0 {
			self.terrain.resize((new_width * self.height) as usize, terrain);
			for y in (0..self.height).rev() {
				let src = y * self.width;
				let dest = y * new_width + additional;
				self.terrain.copy_within(src as usize..(src + self.width) as usize, dest as usize);
				self.terrain[dest as usize..(dest + additional) as usize].fill(terrain);
			}
		}
		else {
			for y in 1..self.height {
				let src = y * self.width;
				let dest = y * new_width;
				self.terrain.copy_within(src as usize..(src + self.width) as usize, dest as usize);
			}
			self.terrain.truncate((new_width * self.height) as usize);
		}
		self.width = new_width;
	}
}

fn clamp_field_size(size: i32) -> i32 {
	cmp::min(255, cmp::max(0, size))
}

impl GameState {
	pub fn resize_field(&mut self, dir: Compass, additional: i32, terrain: Terrain) {
		if additional == 0 {
			return;
		}

		match dir {
			Compass::Up => self.field.resize_top(additional, terrain),
			Compass::Left => self.field.resize_left(additional, terrain),
			Compass::Down => self.field.resize_bottom(additional, terrain),
			Compass::Right => self.field.resize_right(additional, terrain),
		}

		let d = match dir {
			Compass::Down => Vec2i(0, additional),
			Compass::Right => Vec2i(additional, 0),
			_ => Vec2i::ZERO,
		};

		for conn in &mut self.field.conns {
			conn.src += d;
			conn.dest += d;
		}

		for ehandle in self.ents.handles() {
			if let Some(mut ent) = self.ents.take(ehandle) {
				let new_pos = ent.pos + d;
				self.qt.update(ehandle, ent.pos, new_pos);
				ent.pos = new_pos;
				self.events.push(GameEvent::EntityStep { entity: ehandle });
				self.ents.put(ent);
			}
		}
	}
}
