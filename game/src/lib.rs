
pub mod core;
pub mod editor;
pub mod fx;
pub mod menu;
pub mod play;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MusicId {
	Chip1,
	Chip2,
	Canyon,
}
