use super::*;

/// Kinds of an entity.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EntityKind {
	Player,
	PlayerNPC,
	Chip,
	Socket,
	Block,
	IceBlock,
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

impl str::FromStr for EntityKind {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Player" => Ok(EntityKind::Player),
			"PlayerNPC" => Ok(EntityKind::PlayerNPC),
			"Chip" => Ok(EntityKind::Chip),
			"Socket" => Ok(EntityKind::Socket),
			"Block" => Ok(EntityKind::Block),
			"IceBlock" => Ok(EntityKind::IceBlock),
			"Flippers" => Ok(EntityKind::Flippers),
			"FireBoots" => Ok(EntityKind::FireBoots),
			"IceSkates" => Ok(EntityKind::IceSkates),
			"SuctionBoots" => Ok(EntityKind::SuctionBoots),
			"BlueKey" => Ok(EntityKind::BlueKey),
			"RedKey" => Ok(EntityKind::RedKey),
			"GreenKey" => Ok(EntityKind::GreenKey),
			"YellowKey" => Ok(EntityKind::YellowKey),
			"Thief" => Ok(EntityKind::Thief),
			"Bomb" => Ok(EntityKind::Bomb),
			"Bug" => Ok(EntityKind::Bug),
			"FireBall" => Ok(EntityKind::FireBall),
			"PinkBall" => Ok(EntityKind::PinkBall),
			"Tank" => Ok(EntityKind::Tank),
			"Glider" => Ok(EntityKind::Glider),
			"Teeth" => Ok(EntityKind::Teeth),
			"Walker" => Ok(EntityKind::Walker),
			"Blob" => Ok(EntityKind::Blob),
			"Paramecium" => Ok(EntityKind::Paramecium),
			_ => Err("Unknown entity kind"),
		}
	}
}

/// Entity construction arguments.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug)]
pub struct EntityArgs {
	pub kind: EntityKind,
	pub pos: Vec2i,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub face_dir: Option<Compass>,
}
