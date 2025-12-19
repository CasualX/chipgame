use super::*;

pub mod terrain;
pub mod entity;
pub mod connection;
pub mod icepath;
pub mod forcepath;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tool {
	Terrain,
	Entity,
	Connection,
	IcePath,
	ForcePath,
}
impl Default for Tool {
	fn default() -> Self {
		Tool::Terrain
	}
}
