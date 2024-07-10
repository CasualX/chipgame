use super::*;
use editor::*;
use chipgame::core;

pub mod terrain;
pub mod entity;
pub mod connection;

fn next_face_dir(face_dir: Option<core::Compass>) -> Option<core::Compass> {
	match face_dir {
		Some(core::Compass::Up) => Some(core::Compass::Right),
		Some(core::Compass::Right) => Some(core::Compass::Down),
		Some(core::Compass::Down) => Some(core::Compass::Left),
		Some(core::Compass::Left) => None,
		None => Some(core::Compass::Up),
	}
}
