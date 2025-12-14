use super::*;

pub mod terrain;
pub mod entity;
pub mod connection;

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
