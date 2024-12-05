use super::*;

pub mod terrain;
pub mod entity;
pub mod connection;

fn next_face_dir(face_dir: Option<Compass>) -> Option<Compass> {
	match face_dir {
		Some(Compass::Up) => Some(Compass::Right),
		Some(Compass::Right) => Some(Compass::Down),
		Some(Compass::Down) => Some(Compass::Left),
		Some(Compass::Left) => None,
		None => Some(Compass::Up),
	}
}
