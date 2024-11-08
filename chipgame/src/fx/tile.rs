use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TileGfx {
	pub sprite: Sprite,
	pub model: Model,
}

pub static TILES_PLAY: [TileGfx; 43] = [
	TileGfx { sprite: Sprite::Blank, model: Model::Empty }, // Terrain::Blank
	TileGfx { sprite: Sprite::Floor, model: Model::Floor }, // Terrain::Floor
	TileGfx { sprite: Sprite::Wall, model: Model::Wall }, // Terrain::Wall
	TileGfx { sprite: Sprite::Floor, model: Model::Floor }, // Terrain::Socket
	TileGfx { sprite: Sprite::BlueLock, model: Model::Wall }, // Terrain::BlueLock
	TileGfx { sprite: Sprite::RedLock, model: Model::Wall }, // Terrain::RedLock
	TileGfx { sprite: Sprite::GreenLock, model: Model::Wall }, // Terrain::GreenLock
	TileGfx { sprite: Sprite::YellowLock, model: Model::Wall }, // Terrain::YellowLock
	TileGfx { sprite: Sprite::Hint, model: Model::Floor }, // Terrain::Hint
	TileGfx { sprite: Sprite::Exit1, model: Model::Portal }, // Terrain::Exit
	TileGfx { sprite: Sprite::Water, model: Model::Floor }, // Terrain::Water
	TileGfx { sprite: Sprite::Floor, model: Model::Floor }, // Terrain::Fire
	TileGfx { sprite: Sprite::Dirt, model: Model::Floor }, // Terrain::Dirt
	TileGfx { sprite: Sprite::Gravel, model: Model::Floor }, // Terrain::Gravel
	TileGfx { sprite: Sprite::Ice, model: Model::Floor }, // Terrain::Ice
	TileGfx { sprite: Sprite::IceUL, model: Model::Floor }, // Terrain::IceNW
	TileGfx { sprite: Sprite::IceUR, model: Model::Floor }, // Terrain::IceNE
	TileGfx { sprite: Sprite::IceDL, model: Model::Floor }, // Terrain::IceSW
	TileGfx { sprite: Sprite::IceDR, model: Model::Floor }, // Terrain::IceSE
	TileGfx { sprite: Sprite::ForceUp, model: Model::Floor }, // Terrain::ForceN
	TileGfx { sprite: Sprite::ForceLeft, model: Model::Floor }, // Terrain::ForceW
	TileGfx { sprite: Sprite::ForceDown, model: Model::Floor }, // Terrain::ForceS
	TileGfx { sprite: Sprite::ForceRight, model: Model::Floor }, // Terrain::ForceE
	TileGfx { sprite: Sprite::ForceRandom, model: Model::Floor }, // Terrain::ForceRandom
	TileGfx { sprite: Sprite::CloneMachine, model: Model::Wall }, // Terrain::CloneMachine
	TileGfx { sprite: Sprite::OnOffFloor, model: Model::Floor }, // Terrain::ToggleFloor
	TileGfx { sprite: Sprite::OnOffFloor, model: Model::Floor }, // Terrain::ToggleWall
	TileGfx { sprite: Sprite::PanelNorth, model: Model::Floor }, // Terrain::ThinWallN
	TileGfx { sprite: Sprite::PanelWest, model: Model::Floor }, // Terrain::ThinWallW
	TileGfx { sprite: Sprite::PanelSouth, model: Model::Floor }, // Terrain::ThinWallS
	TileGfx { sprite: Sprite::PanelEast, model: Model::Floor }, // Terrain::ThinWallE
	TileGfx { sprite: Sprite::PanelSE, model: Model::Floor }, // Terrain::ThinWallSE
	TileGfx { sprite: Sprite::Floor, model: Model::Floor }, // Terrain::HiddenWall
	TileGfx { sprite: Sprite::Floor, model: Model::Floor }, // Terrain::InvisibleWall
	TileGfx { sprite: Sprite::BlueWall, model: Model::Wall }, // Terrain::RealBlueWall
	TileGfx { sprite: Sprite::BlueWall, model: Model::Wall }, // Terrain::FakeBlueWall
	TileGfx { sprite: Sprite::GreenSwitch, model: Model::Floor }, // Terrain::GreenButton
	TileGfx { sprite: Sprite::RedSwitch, model: Model::Floor }, // Terrain::RedButton
	TileGfx { sprite: Sprite::BrownSwitch, model: Model::Floor }, // Terrain::BrownButton
	TileGfx { sprite: Sprite::BlueSwitch, model: Model::Floor }, // Terrain::BlueButton
	TileGfx { sprite: Sprite::Teleport, model: Model::Floor }, // Terrain::Teleport
	TileGfx { sprite: Sprite::BearTrap, model: Model::Floor }, // Terrain::BearTrap
	TileGfx { sprite: Sprite::RecessedWall, model: Model::Floor }, // Terrain::RecessedWall
];
