/*!
Core game logic.
 */

use std::cmp;
use cvmath::Vec2i;

mod compass;
mod entities;
mod entity;
mod entitymap;
mod event;
mod field;
mod gamestate;
mod inbuf;
mod physics;
mod playerstate;
mod random;
mod terrain;
mod quadtree;

pub use self::compass::*;
pub use self::entities::*;
pub use self::entity::*;
pub use self::entitymap::*;
pub use self::event::*;
pub use self::field::*;
pub use self::gamestate::*;
pub use self::inbuf::*;
pub use self::physics::*;
pub use self::playerstate::*;
pub use self::random::*;
pub use self::terrain::*;
pub use self::quadtree::*;

/// Input data.
#[derive(Copy, Clone, Debug, Default)]
pub struct Input {
	pub a: bool,
	pub b: bool,
	pub left: bool,
	pub right: bool,
	pub up: bool,
	pub down: bool,
}

impl Input {
	pub fn any(&self) -> bool {
		self.a || self.b || self.left || self.right || self.up || self.down
	}
}

#[derive(Copy, Clone, Debug)]
enum TrapState {
	Closed,
	Open,
}

#[derive(Debug)]
pub struct InteractContext {
	pub blocking: bool,
	pub push_dir: Compass,
}

const SOLID_WALL: u8 = 0xf;
const PANEL_N: u8 = 0x1;
const PANEL_E: u8 = 0x2;
const PANEL_S: u8 = 0x4;
const PANEL_W: u8 = 0x8;

type Time = i32;
