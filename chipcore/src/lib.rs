/*!
Core chipgame gameplay code.
*/

use std::cmp;
use cvmath::Vec2i;

mod codes;
mod compass;
mod entities;
mod entity;
mod entitymap;
mod event;
mod field;
mod gamestate;
mod inbuf;
mod input;
mod physics;
mod playerstate;
mod quadtree;
mod random;
mod soundfx;
mod terrain;

pub use self::codes::*;
pub use self::compass::*;
pub use self::entities::*;
pub use self::entity::*;
pub use self::entitymap::*;
pub use self::event::*;
pub use self::field::*;
pub use self::gamestate::*;
pub use self::inbuf::*;
pub use self::input::*;
pub use self::physics::*;
pub use self::playerstate::*;
pub use self::quadtree::*;
pub use self::random::*;
pub use self::soundfx::*;
pub use self::terrain::*;

#[derive(Copy, Clone, Debug)]
enum TrapState {
	Closed,
	Open,
}

const SOLID_WALL: u8 = 0xf;
const THIN_WALL_N: u8 = 0x1;
const THIN_WALL_E: u8 = 0x2;
const THIN_WALL_S: u8 = 0x4;
const THIN_WALL_W: u8 = 0x8;

type Time = i32;
