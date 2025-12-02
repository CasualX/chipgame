use super::*;

pub const FIELD_MIN_WIDTH: i32 = 3;
pub const FIELD_MAX_WIDTH: i32 = 255;
pub const FIELD_MIN_HEIGHT: i32 = 3;
pub const FIELD_MAX_HEIGHT: i32 = 255;

/// Level map data transfer object.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FieldDto {
	pub width: i32,
	pub height: i32,
	pub data: Vec<u8>,
	pub legend: Vec<Terrain>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Trophy {
	Author,
	Gold,
	Silver,
	Bronze,
}

impl Trophy {
	#[inline]
	pub fn to_str(self) -> &'static str {
		match self {
			Trophy::Author => "Author",
			Trophy::Gold => "Gold",
			Trophy::Silver => "Silver",
			Trophy::Bronze => "Bronze",
		}
	}
}

#[derive(Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TrophyValues {
	pub author: i32,
	pub gold: i32,
	pub silver: i32,
	pub bronze: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug)]
pub struct Trophies {
	pub ticks: TrophyValues,
	pub steps: TrophyValues,
}

/// Level data transfer object.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelDto {
	pub name: String,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub author: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub hint: Option<String>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub password: Option<String>,
	pub required_chips: i32,
	#[serde(default, skip_serializing_if = "is_default")]
	pub time_limit: i32,
	pub map: FieldDto,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub entities: Vec<EntityArgs>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub connections: Vec<FieldConn>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub replays: Option<Vec<ReplayDto>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub trophies: Option<Trophies>,
}

/// Connection between terrain tiles.
///
/// The connection defines the relationship between:
/// * A red button and associated clone machine terrain.
/// * A brown button and associated bear trap terrain.
/// * A teleport pad and destination terrain.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct FieldConn {
	pub src: Vec2i,
	pub dest: Vec2i,
}

/// Level brush for editing levels.
#[derive(Clone, Debug)]
pub struct LevelBrush {
	pub width: i32,
	pub height: i32,
	pub terrain: Vec<Option<Terrain>>,
	pub entities: Vec<EntityArgs>,
	pub connections: Vec<FieldConn>,
}
impl LevelBrush {
	#[inline]
	pub fn is_pos_inside(&self, pos: Vec2i) -> bool {
		pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
	}
}
