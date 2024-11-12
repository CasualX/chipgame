use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TileGfx {
	pub sprite: data::SpriteId,
	pub model: data::ModelId,
}

pub static TILES_PLAY: [TileGfx; 50] = [
	TileGfx { sprite: data::SpriteId::Blank, model: data::ModelId::Empty }, // Terrain::Blank
	TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::Floor
	TileGfx { sprite: data::SpriteId::Wall, model: data::ModelId::Wall }, // Terrain::Wall
	TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::Socket
	TileGfx { sprite: data::SpriteId::BlueLock, model: data::ModelId::Wall }, // Terrain::BlueLock
	TileGfx { sprite: data::SpriteId::RedLock, model: data::ModelId::Wall }, // Terrain::RedLock
	TileGfx { sprite: data::SpriteId::GreenLock, model: data::ModelId::Wall }, // Terrain::GreenLock
	TileGfx { sprite: data::SpriteId::YellowLock, model: data::ModelId::Wall }, // Terrain::YellowLock
	TileGfx { sprite: data::SpriteId::Hint, model: data::ModelId::Floor }, // Terrain::Hint
	TileGfx { sprite: data::SpriteId::Exit1, model: data::ModelId::Portal }, // Terrain::Exit
	TileGfx { sprite: data::SpriteId::Exit1, model: data::ModelId::Portal }, // Terrain::FakeExit
	TileGfx { sprite: data::SpriteId::Water, model: data::ModelId::Floor }, // Terrain::Water
	TileGfx { sprite: data::SpriteId::WaterSplash, model: data::ModelId::Floor }, // Terrain::WaterHazard
	TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::Fire
	TileGfx { sprite: data::SpriteId::Dirt, model: data::ModelId::Floor }, // Terrain::Dirt
	TileGfx { sprite: data::SpriteId::DirtBlock, model: data::ModelId::Wall }, // Terrain::DirtBlock
	TileGfx { sprite: data::SpriteId::Gravel, model: data::ModelId::Floor }, // Terrain::Gravel
	TileGfx { sprite: data::SpriteId::Ice, model: data::ModelId::Floor }, // Terrain::Ice
	TileGfx { sprite: data::SpriteId::IceCornerNW, model: data::ModelId::Floor }, // Terrain::IceNW
	TileGfx { sprite: data::SpriteId::IceCornerNE, model: data::ModelId::Floor }, // Terrain::IceNE
	TileGfx { sprite: data::SpriteId::IceCornerSW, model: data::ModelId::Floor }, // Terrain::IceSW
	TileGfx { sprite: data::SpriteId::IceCornerSE, model: data::ModelId::Floor }, // Terrain::IceSE
	TileGfx { sprite: data::SpriteId::ForceUp, model: data::ModelId::Floor }, // Terrain::ForceN
	TileGfx { sprite: data::SpriteId::ForceLeft, model: data::ModelId::Floor }, // Terrain::ForceW
	TileGfx { sprite: data::SpriteId::ForceDown, model: data::ModelId::Floor }, // Terrain::ForceS
	TileGfx { sprite: data::SpriteId::ForceRight, model: data::ModelId::Floor }, // Terrain::ForceE
	TileGfx { sprite: data::SpriteId::ForceRandom, model: data::ModelId::Floor }, // Terrain::ForceRandom
	TileGfx { sprite: data::SpriteId::CloneMachine, model: data::ModelId::Wall }, // Terrain::CloneMachine
	TileGfx { sprite: data::SpriteId::CloneBlockN, model: data::ModelId::Wall }, // Terrain::CloneBlockN
	TileGfx { sprite: data::SpriteId::CloneBlockW, model: data::ModelId::Wall }, // Terrain::CloneBlockW
	TileGfx { sprite: data::SpriteId::CloneBlockS, model: data::ModelId::Wall }, // Terrain::CloneBlockS
	TileGfx { sprite: data::SpriteId::CloneBlockE, model: data::ModelId::Wall }, // Terrain::CloneBlockE
	TileGfx { sprite: data::SpriteId::OnOffFloor, model: data::ModelId::Floor }, // Terrain::ToggleFloor
	TileGfx { sprite: data::SpriteId::OnOffFloor, model: data::ModelId::Floor }, // Terrain::ToggleWall
	TileGfx { sprite: data::SpriteId::ThinWallN, model: data::ModelId::Floor }, // Terrain::ThinWallN
	TileGfx { sprite: data::SpriteId::ThinWallW, model: data::ModelId::Floor }, // Terrain::ThinWallW
	TileGfx { sprite: data::SpriteId::ThinWallS, model: data::ModelId::Floor }, // Terrain::ThinWallS
	TileGfx { sprite: data::SpriteId::ThinWallE, model: data::ModelId::Floor }, // Terrain::ThinWallE
	TileGfx { sprite: data::SpriteId::ThinWallSE, model: data::ModelId::Floor }, // Terrain::ThinWallSE
	TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::HiddenWall
	TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::InvisibleWall
	TileGfx { sprite: data::SpriteId::BlueWall, model: data::ModelId::Wall }, // Terrain::RealBlueWall
	TileGfx { sprite: data::SpriteId::BlueWall, model: data::ModelId::Wall }, // Terrain::FakeBlueWall
	TileGfx { sprite: data::SpriteId::GreenSwitch, model: data::ModelId::Floor }, // Terrain::GreenButton
	TileGfx { sprite: data::SpriteId::RedSwitch, model: data::ModelId::Floor }, // Terrain::RedButton
	TileGfx { sprite: data::SpriteId::BrownSwitch, model: data::ModelId::Floor }, // Terrain::BrownButton
	TileGfx { sprite: data::SpriteId::BlueSwitch, model: data::ModelId::Floor }, // Terrain::BlueButton
	TileGfx { sprite: data::SpriteId::Teleport, model: data::ModelId::Floor }, // Terrain::Teleport
	TileGfx { sprite: data::SpriteId::BearTrap, model: data::ModelId::Floor }, // Terrain::BearTrap
	TileGfx { sprite: data::SpriteId::RecessedWall, model: data::ModelId::Floor }, // Terrain::RecessedWall
];
