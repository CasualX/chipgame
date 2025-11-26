//! Level editor.

use std::collections::HashMap;
use std::mem;
use cvmath::*;
use chipcore::EntityHandle;
use chipty::{Compass, EntityArgs, EntityKind, FieldConn, FieldDto, LevelDto, Terrain};

use crate::fx;
use crate::data;
use crate::render;
use crate::menu;

mod tool;
mod tiles;
mod edit;
mod play;

use self::edit::EditorEditState;
use self::play::EditorPlayState;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tool {
	Terrain,
	Entity,
	Connection,
}
impl Default for Tool {
	fn default() -> Self {
		Tool::Terrain
	}
}

static TERRAIN_SAMPLES: [[Terrain; 2]; 24] = [
	[Terrain::Blank, Terrain::Floor],
	[Terrain::Dirt, Terrain::Gravel],
	[Terrain::Wall, Terrain::CloneMachine],
	[Terrain::HiddenWall, Terrain::InvisibleWall],
	[Terrain::RealBlueWall, Terrain::FakeBlueWall],
	[Terrain::BlueLock, Terrain::RedLock],
	[Terrain::GreenLock, Terrain::YellowLock],
	[Terrain::Exit, Terrain::Hint],
	[Terrain::Water, Terrain::Fire],
	[Terrain::WaterHazard, Terrain::DirtBlock],
	[Terrain::ThinWallE, Terrain::ThinWallS],
	[Terrain::ThinWallN, Terrain::ThinWallW],
	[Terrain::ThinWallSE, Terrain::Ice],
	[Terrain::IceNW, Terrain::IceNE],
	[Terrain::IceSW, Terrain::IceSE],
	[Terrain::ToggleFloor, Terrain::ToggleWall],
	[Terrain::GreenButton, Terrain::RedButton],
	[Terrain::BrownButton, Terrain::BlueButton],
	[Terrain::BearTrap, Terrain::RecessedWall],
	[Terrain::Teleport, Terrain::ForceRandom],
	[Terrain::ForceE, Terrain::ForceS],
	[Terrain::ForceN, Terrain::ForceW],
	[Terrain::CloneBlockE, Terrain::CloneBlockS],
	[Terrain::CloneBlockN, Terrain::CloneBlockW],
];

static ENTITY_SAMPLES: [(EntityKind, data::SpriteId); 24] = [
	(EntityKind::Player, data::SpriteId::PlayerWalkNeutral),
	(EntityKind::Chip, data::SpriteId::Chip),
	(EntityKind::Socket, data::SpriteId::Socket),
	(EntityKind::Block, data::SpriteId::DirtBlock),
	(EntityKind::IceBlock, data::SpriteId::IceBlock),
	(EntityKind::Flippers, data::SpriteId::Flippers),
	(EntityKind::FireBoots, data::SpriteId::FireBoots),
	(EntityKind::IceSkates, data::SpriteId::IceSkates),
	(EntityKind::SuctionBoots, data::SpriteId::SuctionBoots),
	(EntityKind::BlueKey, data::SpriteId::BlueKey),
	(EntityKind::RedKey, data::SpriteId::RedKey),
	(EntityKind::GreenKey, data::SpriteId::GreenKey),
	(EntityKind::YellowKey, data::SpriteId::YellowKey),
	(EntityKind::Thief, data::SpriteId::Thief),
	(EntityKind::Bomb, data::SpriteId::Bomb),
	(EntityKind::Bug, data::SpriteId::BugUp),
	(EntityKind::FireBall, data::SpriteId::FireBall),
	(EntityKind::PinkBall, data::SpriteId::PinkBall),
	(EntityKind::Tank, data::SpriteId::TankUp),
	(EntityKind::Glider, data::SpriteId::GliderUp),
	(EntityKind::Teeth, data::SpriteId::TeethUp),
	(EntityKind::Walker, data::SpriteId::WalkerUpDown),
	(EntityKind::Blob, data::SpriteId::Blob),
	(EntityKind::Paramecium, data::SpriteId::ParameciumUpDown),
];

#[derive(Default)]
pub struct Input {
	pub left_click: bool,
	pub right_click: bool,
	pub key_left: bool,
	pub key_right: bool,
	pub key_up: bool,
	pub key_down: bool,
}

pub enum EditorState {
	Edit(Box<EditorEditState>),
	Play(Box<EditorPlayState>),
}

impl Default for EditorState {
	fn default() -> Self {
		EditorState::Edit(Box::new(EditorEditState::default()))
	}
}

impl EditorState {
	pub fn init(&mut self) {
		match self {
			EditorState::Edit(s) => s.init(),
			EditorState::Play(s) => s.init(),
		}
	}
	pub fn load_level(&mut self, json: &str) {
		match self {
			EditorState::Edit(s) => s.load_level(json),
			EditorState::Play(_) => {}
		}
	}
	pub fn reload_level(&mut self, json: &str) {
		match self {
			EditorState::Edit(s) => s.reload_level(json),
			EditorState::Play(_) => {}
		}
	}
	pub fn save_level(&self) -> String {
		match self {
			EditorState::Edit(s) => s.save_level(),
			EditorState::Play(s) => s.level.clone(),
		}
	}
	pub fn set_screen_size(&mut self, width: i32, height: i32) {
		match self {
			EditorState::Edit(s) => s.set_screen_size(width, height),
			EditorState::Play(s) => s.set_screen_size(width, height),
		}
	}
	pub fn mouse_move(&mut self, mouse_x: i32, mouse_y: i32) {
		match self {
			EditorState::Edit(s) => s.mouse_move(mouse_x, mouse_y),
			EditorState::Play(_) => (),
		}
	}
	pub fn key_left(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.key_left(pressed),
			EditorState::Play(s) => s.key_left(pressed),
		}
	}
	pub fn key_right(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.key_right(pressed),
			EditorState::Play(s) => s.key_right(pressed),
		}
	}
	pub fn key_up(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.key_up(pressed),
			EditorState::Play(s) => s.key_up(pressed),
		}
	}
	pub fn key_down(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.key_down(pressed),
			EditorState::Play(s) => s.key_down(pressed),
		}
	}
	pub fn toggle_play(&mut self) {
		match self {
			EditorState::Edit(s) => {
				let level = s.save_level();
				let level_dto = serde_json::from_str(&level).unwrap();
				let mut game = fx::FxState::default();
				game.render.tiles = &crate::play::tiles::TILES_PLAY;
				game.parse_level(0, &level_dto, chipcore::RngSeed::System);
				game.camera.set_perspective(true);
				*self = EditorState::Play(Box::new(EditorPlayState {
					level,
					game,
					input: Input::default(),
					screen_size: s.screen_size,
				}));
			}
			EditorState::Play(s) => {
				let mut state = EditorEditState::default();
				state.init();
				state.load_level(&s.level);
				state.screen_size = s.screen_size;
				*self = EditorState::Edit(Box::new(state));
			},
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources) {
		match self {
			EditorState::Edit(s) => s.draw(g, resx),
			EditorState::Play(s) => s.draw(g, resx),
		}
	}

	pub fn take_fx_events(&mut self) -> Vec<fx::FxEvent> {
		match self {
			EditorState::Edit(s) => mem::take(&mut s.game.events),
			EditorState::Play(s) => mem::take(&mut s.game.events),
		}
	}

	pub fn tool_terrain(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.tool_terrain(pressed),
			EditorState::Play(_) => {},
		}
	}
	pub fn tool_entity(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.tool_entity(pressed),
			EditorState::Play(_) => {},
		}
	}
	pub fn tool_connection(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.tool_connection(pressed),
			EditorState::Play(_) => {},
		}
	}

	pub fn resize(&mut self, left: i32, top: i32, right: i32, bottom: i32) {
		match self {
			EditorState::Edit(s) => s.resize(left, top, right, bottom),
			EditorState::Play(_) => {},
		}
	}

	pub fn expand_top(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(0, 1, 0, 0),
			EditorState::Play(_) => {},
		}
	}
	pub fn expand_bottom(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(0, 0, 0, 1),
			EditorState::Play(_) => {},
		}
	}
	pub fn expand_left(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(1, 0, 0, 0),
			EditorState::Play(_) => {},
		}
	}
	pub fn expand_right(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(0, 0, 1, 0),
			EditorState::Play(_) => {},
		}
	}
	pub fn crop_top(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(0, -1, 0, 0),
			EditorState::Play(_) => {},
		}
	}
	pub fn crop_bottom(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(0, 0, 0, -1),
			EditorState::Play(_) => {},
		}
	}
	pub fn crop_left(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(-1, 0, 0, 0),
			EditorState::Play(_) => {},
		}
	}
	pub fn crop_right(&mut self) {
		match self {
			EditorState::Edit(s) => s.resize(0, 0, -1, 0),
			EditorState::Play(_) => {},
		}
	}

	pub fn left_click(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.left_click(pressed),
			EditorState::Play(_) => {},
		}
	}
	pub fn right_click(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.right_click(pressed),
			EditorState::Play(_) => {},
		}
	}
	pub fn delete(&mut self, pressed: bool) {
		match self {
			EditorState::Edit(s) => s.delete(pressed),
			EditorState::Play(_) => {},
		}
	}

	pub fn sample(&mut self) {
		match self {
			EditorState::Edit(s) => s.sample(),
			EditorState::Play(_) => {},
		}
	}

	pub fn get_tool(&self) -> Option<Tool> {
		match self {
			EditorState::Edit(s) => Some(s.tool),
			EditorState::Play(_) => None,
		}
	}
}
