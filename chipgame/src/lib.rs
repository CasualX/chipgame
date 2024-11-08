
// pub mod core;
pub use chipcore as core;

pub mod editor;
pub mod fx;
pub mod menu;
pub mod play;
pub mod save;
pub mod gui;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MusicId {
	Chip1,
	Chip2,
	Canyon,
}

// pub const LEVEL_PACK: &str = "data/packs/cclp1/";
