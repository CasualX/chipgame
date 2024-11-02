
// pub mod core;
pub use chipgameplay as core;

pub mod editor;
pub mod fx;
pub mod menu;
pub mod play;
pub mod save;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MusicId {
	Chip1,
	Chip2,
	Canyon,
}

pub const LEVEL_PACK: &str = "data/cclp1/";
