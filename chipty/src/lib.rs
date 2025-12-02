use std::collections::HashMap;
use std::str;
use cvmath::*;

mod compass;
mod compress;
mod entity;
mod level;
mod levelset;
mod music;
mod replay;
mod savedata;
mod soundfx;
mod spritesheet;
mod terrain;

pub use compass::*;
pub use compress::*;
pub use entity::*;
pub use level::*;
pub use levelset::*;
pub use music::*;
pub use replay::*;
pub use savedata::*;
pub use soundfx::*;
pub use spritesheet::*;
pub use terrain::*;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
	*value == T::default()
}
