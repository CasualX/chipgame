use super::*;

/// Terrain types for the game.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Terrain {
	#[default]
	Blank,
	Floor,
	Wall,
	Socket,
	BlueLock,
	RedLock,
	GreenLock,
	YellowLock,
	Hint,
	Exit,
	Water,
	Fire,
	Dirt,
	Gravel,
	Ice,
	IceNW,
	IceNE,
	IceSW,
	IceSE,
	ForceN,
	ForceW,
	ForceS,
	ForceE,
	ForceRandom,
	CloneMachine,
	ToggleFloor,
	ToggleWall,
	ThinWallN,
	ThinWallW,
	ThinWallS,
	ThinWallE,
	ThinWallSE,
	HiddenWall,
	InvisibleWall,
	RealBlueWall,
	FakeBlueWall,
	GreenButton,
	RedButton,
	BrownButton,
	BlueButton,
	Teleport,
	BearTrap,
	RecessedWall,
}

impl Terrain {
	pub fn solid_flags(self) -> u8 {
		match self {
			Terrain::Blank => SOLID_WALL,
			Terrain::Floor => 0,
			Terrain::Wall => SOLID_WALL,
			Terrain::Socket => SOLID_WALL,
			Terrain::BlueLock => SOLID_WALL,
			Terrain::RedLock => SOLID_WALL,
			Terrain::GreenLock => SOLID_WALL,
			Terrain::YellowLock => SOLID_WALL,
			Terrain::Hint => 0,
			Terrain::Exit => 0,
			Terrain::Water => 0,
			Terrain::Fire => 0,
			Terrain::Dirt => 0,
			Terrain::Gravel => 0,
			Terrain::Ice => 0,
			Terrain::IceNW => THIN_WALL_N | THIN_WALL_W,
			Terrain::IceNE => THIN_WALL_N | THIN_WALL_E,
			Terrain::IceSW => THIN_WALL_S | THIN_WALL_W,
			Terrain::IceSE => THIN_WALL_S | THIN_WALL_E,
			Terrain::ForceN => 0,
			Terrain::ForceW => 0,
			Terrain::ForceS => 0,
			Terrain::ForceE => 0,
			Terrain::ForceRandom => 0,
			Terrain::CloneMachine => SOLID_WALL,
			Terrain::ToggleFloor => 0,
			Terrain::ToggleWall => SOLID_WALL,
			Terrain::ThinWallN => THIN_WALL_N,
			Terrain::ThinWallW => THIN_WALL_W,
			Terrain::ThinWallS => THIN_WALL_S,
			Terrain::ThinWallE => THIN_WALL_E,
			Terrain::ThinWallSE => THIN_WALL_S | THIN_WALL_E,
			Terrain::HiddenWall => SOLID_WALL,
			Terrain::InvisibleWall => SOLID_WALL,
			Terrain::RealBlueWall => SOLID_WALL,
			Terrain::FakeBlueWall => 0,
			Terrain::GreenButton => 0,
			Terrain::RedButton => 0,
			Terrain::BrownButton => 0,
			Terrain::BlueButton => 0,
			Terrain::Teleport => 0,
			Terrain::BearTrap => 0,
			Terrain::RecessedWall => 0,
		}
	}
}
