use super::*;

mod terrain;
mod entity;
mod connection;
mod icepath;
mod forcepath;

pub use self::terrain::TerrainToolState;
pub use self::entity::EntityToolState;
pub use self::connection::ConnectionToolState;
pub use self::icepath::IcePathToolState;
pub use self::forcepath::ForcePathToolState;

pub enum ToolState {
	Terrain(TerrainToolState),
	Entity(EntityToolState),
	Connection(ConnectionToolState),
	IcePath(IcePathToolState),
	ForcePath(ForcePathToolState),
}
impl ToolState {
	pub fn name(&self) -> &'static str {
		match self {
			ToolState::Terrain(_) => "Terrain",
			ToolState::Entity(_) => "Entity",
			ToolState::Connection(_) => "Connection",
			ToolState::IcePath(_) => "Ice Path",
			ToolState::ForcePath(_) => "Force Path",
		}
	}

	pub fn left_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		match self {
			ToolState::Terrain(state) => state.left_click(s, pressed),
			ToolState::Entity(state) => state.left_click(s, pressed),
			ToolState::Connection(state) => state.left_click(s, pressed),
			ToolState::IcePath(state) => state.left_click(s, pressed),
			ToolState::ForcePath(state) => state.left_click(s, pressed),
		}
	}

	pub fn right_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		match self {
			ToolState::Terrain(state) => state.right_click(s, pressed),
			ToolState::Entity(state) => state.right_click(s, pressed),
			ToolState::Connection(state) => state.right_click(s, pressed),
			ToolState::IcePath(state) => state.right_click(s, pressed),
			ToolState::ForcePath(state) => state.right_click(s, pressed),
		}
	}

	pub fn think(&mut self, s: &mut EditorEditState) {
		match self {
			ToolState::Terrain(state) => state.think(s),
			ToolState::Entity(state) => state.think(s),
			ToolState::Connection(state) => state.think(s),
			ToolState::IcePath(state) => state.think(s),
			ToolState::ForcePath(state) => state.think(s),
		}
	}

	pub fn delete(&mut self, s: &mut EditorEditState, pressed: bool) {
		if let ToolState::Entity(state) = self {
			state.delete(s, pressed);
		}
	}
}
