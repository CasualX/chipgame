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
mod quadtree;
mod random;
mod soundfx;
mod terrain;

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
pub use self::quadtree::*;
pub use self::random::*;
pub use self::soundfx::*;
pub use self::terrain::*;

/// Input data.
#[derive(Copy, Clone, Debug, Default)]
pub struct Input {
	pub up: bool,
	pub left: bool,
	pub down: bool,
	pub right: bool,
	pub a: bool,
	pub b: bool,
}

impl Input {
	pub fn any(&self) -> bool {
		self.a || self.b || self.left || self.right || self.up || self.down
	}

	pub fn encode(&self, buf: &mut Vec<u8>) {
		let mut bits = 0;
		if self.up {
			bits |= INPUT_UP;
		}
		if self.left {
			bits |= INPUT_LEFT;
		}
		if self.down {
			bits |= INPUT_DOWN;
		}
		if self.right {
			bits |= INPUT_RIGHT;
		}
		if self.a {
			bits |= INPUT_A;
		}
		if self.b {
			bits |= INPUT_B;
		}
		buf.push(bits);
	}
}

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_LEFT: u8 = 1 << 1;
pub const INPUT_DOWN: u8 = 1 << 2;
pub const INPUT_RIGHT: u8 = 1 << 3;
pub const INPUT_A: u8 = 1 << 4;
pub const INPUT_B: u8 = 1 << 5;

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
