use super::*;

pub static TILES_EDIT: [fx::TileGfx; 43] = [
	fx::TileGfx { sprite: fx::Sprite::Blank, model: fx::Model::Empty }, // Terrain::Blank
	fx::TileGfx { sprite: fx::Sprite::Floor, model: fx::Model::Floor }, // Terrain::Floor
	fx::TileGfx { sprite: fx::Sprite::Wall, model: fx::Model::Wall }, // Terrain::Wall
	fx::TileGfx { sprite: fx::Sprite::Floor, model: fx::Model::Floor }, // Terrain::Socket
	fx::TileGfx { sprite: fx::Sprite::BlueLock, model: fx::Model::Wall }, // Terrain::BlueLock
	fx::TileGfx { sprite: fx::Sprite::RedLock, model: fx::Model::Wall }, // Terrain::RedLock
	fx::TileGfx { sprite: fx::Sprite::GreenLock, model: fx::Model::Wall }, // Terrain::GreenLock
	fx::TileGfx { sprite: fx::Sprite::YellowLock, model: fx::Model::Wall }, // Terrain::YellowLock
	fx::TileGfx { sprite: fx::Sprite::Hint, model: fx::Model::Floor }, // Terrain::Hint
	fx::TileGfx { sprite: fx::Sprite::Exit1, model: fx::Model::Portal }, // Terrain::Exit
	fx::TileGfx { sprite: fx::Sprite::Water, model: fx::Model::Floor }, // Terrain::Water
	fx::TileGfx { sprite: fx::Sprite::Floor, model: fx::Model::Floor }, // Terrain::Fire
	fx::TileGfx { sprite: fx::Sprite::Dirt, model: fx::Model::Floor }, // Terrain::Dirt
	fx::TileGfx { sprite: fx::Sprite::Gravel, model: fx::Model::Floor }, // Terrain::Gravel
	fx::TileGfx { sprite: fx::Sprite::Ice, model: fx::Model::Floor }, // Terrain::Ice
	fx::TileGfx { sprite: fx::Sprite::IceUL, model: fx::Model::Floor }, // Terrain::IceNW
	fx::TileGfx { sprite: fx::Sprite::IceUR, model: fx::Model::Floor }, // Terrain::IceNE
	fx::TileGfx { sprite: fx::Sprite::IceDL, model: fx::Model::Floor }, // Terrain::IceSW
	fx::TileGfx { sprite: fx::Sprite::IceDR, model: fx::Model::Floor }, // Terrain::IceSE
	fx::TileGfx { sprite: fx::Sprite::ForceUp, model: fx::Model::Floor }, // Terrain::ForceN
	fx::TileGfx { sprite: fx::Sprite::ForceLeft, model: fx::Model::Floor }, // Terrain::ForceW
	fx::TileGfx { sprite: fx::Sprite::ForceDown, model: fx::Model::Floor }, // Terrain::ForceS
	fx::TileGfx { sprite: fx::Sprite::ForceRight, model: fx::Model::Floor }, // Terrain::ForceE
	fx::TileGfx { sprite: fx::Sprite::ForceRandom, model: fx::Model::Floor }, // Terrain::ForceRandom
	fx::TileGfx { sprite: fx::Sprite::CloneMachine, model: fx::Model::Wall }, // Terrain::CloneMachine
	fx::TileGfx { sprite: fx::Sprite::OnOffFloor, model: fx::Model::Floor }, // Terrain::ToggleFloor
	fx::TileGfx { sprite: fx::Sprite::OnOffWall, model: fx::Model::Wall }, // Terrain::ToggleWall
	fx::TileGfx { sprite: fx::Sprite::PanelNorth, model: fx::Model::Floor }, // Terrain::ThinWallN
	fx::TileGfx { sprite: fx::Sprite::PanelWest, model: fx::Model::Floor }, // Terrain::ThinWallW
	fx::TileGfx { sprite: fx::Sprite::PanelSouth, model: fx::Model::Floor }, // Terrain::ThinWallS
	fx::TileGfx { sprite: fx::Sprite::PanelEast, model: fx::Model::Floor }, // Terrain::ThinWallE
	fx::TileGfx { sprite: fx::Sprite::PanelSE, model: fx::Model::Floor }, // Terrain::ThinWallSE
	fx::TileGfx { sprite: fx::Sprite::HiddenWall, model: fx::Model::Wall }, // Terrain::HiddenWall
	fx::TileGfx { sprite: fx::Sprite::InvisWall, model: fx::Model::Wall }, // Terrain::InvisibleWall
	fx::TileGfx { sprite: fx::Sprite::BlueWall, model: fx::Model::Wall }, // Terrain::RealBlueWall
	fx::TileGfx { sprite: fx::Sprite::BlueWallFake, model: fx::Model::Wall }, // Terrain::FakeBlueWall
	fx::TileGfx { sprite: fx::Sprite::GreenSwitch, model: fx::Model::Floor }, // Terrain::GreenButton
	fx::TileGfx { sprite: fx::Sprite::RedSwitch, model: fx::Model::Floor }, // Terrain::RedButton
	fx::TileGfx { sprite: fx::Sprite::BrownSwitch, model: fx::Model::Floor }, // Terrain::BrownButton
	fx::TileGfx { sprite: fx::Sprite::BlueSwitch, model: fx::Model::Floor }, // Terrain::BlueButton
	fx::TileGfx { sprite: fx::Sprite::Teleport, model: fx::Model::Floor }, // Terrain::Teleport
	fx::TileGfx { sprite: fx::Sprite::BearTrap, model: fx::Model::Floor }, // Terrain::BearTrap
	fx::TileGfx { sprite: fx::Sprite::RecessedWall, model: fx::Model::Floor }, // Terrain::RecessedWall
];
