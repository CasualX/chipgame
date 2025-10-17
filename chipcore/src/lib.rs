/*!
Core chipgame gameplay code.
*/

use std::{cmp, ops};
use cvmath::Vec2i;
use chipty::*;

mod codes;
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

pub use self::codes::*;
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

#[derive(Copy, Clone, Debug)]
pub enum TrapState {
	Closed,
	Open,
}

pub enum RngSeed {
	System,
	Manual(u64),
}

type Time = i32;
