use super::*;

pub static TILES_EDIT: [fx::TileGfx; 50] = [
	fx::TileGfx { sprite: data::SpriteId::Blank, model: data::ModelId::Empty }, // Terrain::Blank
	fx::TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::Floor
	fx::TileGfx { sprite: data::SpriteId::Wall, model: data::ModelId::Wall }, // Terrain::Wall
	fx::TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::Socket
	fx::TileGfx { sprite: data::SpriteId::BlueLock, model: data::ModelId::Wall }, // Terrain::BlueLock
	fx::TileGfx { sprite: data::SpriteId::RedLock, model: data::ModelId::Wall }, // Terrain::RedLock
	fx::TileGfx { sprite: data::SpriteId::GreenLock, model: data::ModelId::Wall }, // Terrain::GreenLock
	fx::TileGfx { sprite: data::SpriteId::YellowLock, model: data::ModelId::Wall }, // Terrain::YellowLock
	fx::TileGfx { sprite: data::SpriteId::Hint, model: data::ModelId::Floor }, // Terrain::Hint
	fx::TileGfx { sprite: data::SpriteId::Exit1, model: data::ModelId::Portal }, // Terrain::Exit
	fx::TileGfx { sprite: data::SpriteId::Exit1, model: data::ModelId::Portal }, // Terrain::FakeExit
	fx::TileGfx { sprite: data::SpriteId::Water, model: data::ModelId::Floor }, // Terrain::Water
	fx::TileGfx { sprite: data::SpriteId::WaterSplash, model: data::ModelId::Floor }, // Terrain::WaterHazard
	fx::TileGfx { sprite: data::SpriteId::Floor, model: data::ModelId::Floor }, // Terrain::Fire
	fx::TileGfx { sprite: data::SpriteId::Dirt, model: data::ModelId::Floor }, // Terrain::Dirt
	fx::TileGfx { sprite: data::SpriteId::DirtBlock, model: data::ModelId::Wall }, // Terrain::DirtBlock
	fx::TileGfx { sprite: data::SpriteId::Gravel, model: data::ModelId::Floor }, // Terrain::Gravel
	fx::TileGfx { sprite: data::SpriteId::Ice, model: data::ModelId::Floor }, // Terrain::Ice
	fx::TileGfx { sprite: data::SpriteId::IceCornerNW, model: data::ModelId::Floor }, // Terrain::IceNW
	fx::TileGfx { sprite: data::SpriteId::IceCornerNE, model: data::ModelId::Floor }, // Terrain::IceNE
	fx::TileGfx { sprite: data::SpriteId::IceCornerSW, model: data::ModelId::Floor }, // Terrain::IceSW
	fx::TileGfx { sprite: data::SpriteId::IceCornerSE, model: data::ModelId::Floor }, // Terrain::IceSE
	fx::TileGfx { sprite: data::SpriteId::ForceUp, model: data::ModelId::Floor }, // Terrain::ForceN
	fx::TileGfx { sprite: data::SpriteId::ForceLeft, model: data::ModelId::Floor }, // Terrain::ForceW
	fx::TileGfx { sprite: data::SpriteId::ForceDown, model: data::ModelId::Floor }, // Terrain::ForceS
	fx::TileGfx { sprite: data::SpriteId::ForceRight, model: data::ModelId::Floor }, // Terrain::ForceE
	fx::TileGfx { sprite: data::SpriteId::ForceRandom, model: data::ModelId::Floor }, // Terrain::ForceRandom
	fx::TileGfx { sprite: data::SpriteId::CloneMachine, model: data::ModelId::Wall }, // Terrain::CloneMachine
	fx::TileGfx { sprite: data::SpriteId::CloneBlockN, model: data::ModelId::Wall }, // Terrain::CloneBlockN
	fx::TileGfx { sprite: data::SpriteId::CloneBlockW, model: data::ModelId::Wall }, // Terrain::CloneBlockW
	fx::TileGfx { sprite: data::SpriteId::CloneBlockS, model: data::ModelId::Wall }, // Terrain::CloneBlockS
	fx::TileGfx { sprite: data::SpriteId::CloneBlockE, model: data::ModelId::Wall }, // Terrain::CloneBlockE
	fx::TileGfx { sprite: data::SpriteId::OnOffFloor, model: data::ModelId::Floor }, // Terrain::ToggleFloor
	fx::TileGfx { sprite: data::SpriteId::OnOffWall, model: data::ModelId::Wall }, // Terrain::ToggleWall
	fx::TileGfx { sprite: data::SpriteId::ThinWallN, model: data::ModelId::Floor }, // Terrain::ThinWallN
	fx::TileGfx { sprite: data::SpriteId::ThinWallW, model: data::ModelId::Floor }, // Terrain::ThinWallW
	fx::TileGfx { sprite: data::SpriteId::ThinWallS, model: data::ModelId::Floor }, // Terrain::ThinWallS
	fx::TileGfx { sprite: data::SpriteId::ThinWallE, model: data::ModelId::Floor }, // Terrain::ThinWallE
	fx::TileGfx { sprite: data::SpriteId::ThinWallSE, model: data::ModelId::Floor }, // Terrain::ThinWallSE
	fx::TileGfx { sprite: data::SpriteId::HiddenWall, model: data::ModelId::Wall }, // Terrain::HiddenWall
	fx::TileGfx { sprite: data::SpriteId::InvisWall, model: data::ModelId::Wall }, // Terrain::InvisibleWall
	fx::TileGfx { sprite: data::SpriteId::BlueWall, model: data::ModelId::Wall }, // Terrain::RealBlueWall
	fx::TileGfx { sprite: data::SpriteId::BlueWallFake, model: data::ModelId::Wall }, // Terrain::FakeBlueWall
	fx::TileGfx { sprite: data::SpriteId::GreenSwitch, model: data::ModelId::Floor }, // Terrain::GreenButton
	fx::TileGfx { sprite: data::SpriteId::RedSwitch, model: data::ModelId::Floor }, // Terrain::RedButton
	fx::TileGfx { sprite: data::SpriteId::BrownSwitch, model: data::ModelId::Floor }, // Terrain::BrownButton
	fx::TileGfx { sprite: data::SpriteId::BlueSwitch, model: data::ModelId::Floor }, // Terrain::BlueButton
	fx::TileGfx { sprite: data::SpriteId::Teleport, model: data::ModelId::Floor }, // Terrain::Teleport
	fx::TileGfx { sprite: data::SpriteId::BearTrap, model: data::ModelId::Floor }, // Terrain::BearTrap
	fx::TileGfx { sprite: data::SpriteId::RecessedWall, model: data::ModelId::Floor }, // Terrain::RecessedWall
];
