use super::*;

pub use chipty::EntityKind;

/// Static entity data.
#[derive(Debug)]
pub struct EntityData {
	pub think: fn(&mut GameState, &mut Entity),
	pub flags: SolidFlags,
}

/// Entity structure.
#[derive(Clone, Debug)]
pub struct Entity {
	pub data: &'static EntityData,
	pub handle: EntityHandle,
	pub kind: EntityKind,
	pub pos: Vec2i,
	pub base_spd: Time,
	pub face_dir: Option<Compass>,
	pub step_dir: Option<Compass>,
	pub step_spd: Time,
	pub step_time: Time,
	pub flags: u8,
}

impl Entity {
	pub fn to_entity_args(&self) -> EntityArgs {
		EntityArgs {
			kind: self.kind,
			pos: self.pos,
			face_dir: self.face_dir,
		}
	}
}

/// Entity will be removed at the end of the current tick.
pub const EF_REMOVE: u8 = 1 << 0;
/// Entity is trapped and cannot move.
pub const EF_TRAPPED: u8 = 1 << 1;
/// Entity is hidden under a block.
pub const EF_HIDDEN: u8 = 1 << 2;
/// Entity has a button pressed.
pub const EF_BUTTON_DOWN: u8 = 1 << 3;
/// Entity is a template for cloning.
pub const EF_TEMPLATE: u8 = 1 << 4;
/// Entity has been forced to move.
pub const EF_MOMENTUM: u8 = 1 << 5;
/// Entity has a new position after try_move.
pub const EF_NEW_POS: u8 = 1 << 6;
