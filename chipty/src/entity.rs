use super::*;

/// Kinds of an entity.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityKind {
	Player,
	Chip,
	Socket,
	Block,
	Flippers,
	FireBoots,
	IceSkates,
	SuctionBoots,
	BlueKey,
	RedKey,
	GreenKey,
	YellowKey,
	Thief,
	Bomb,
	Bug,
	FireBall,
	PinkBall,
	Tank,
	Glider,
	Teeth,
	Walker,
	Blob,
	Paramecium,
}

/// Entity construction arguments.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug)]
pub struct EntityArgs {
	pub kind: EntityKind,
	pub pos: Vec2i,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub face_dir: Option<Compass>,
}
