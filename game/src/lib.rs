
pub mod core;
pub mod fx;
pub mod editor;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MusicId {
	Chip1,
	Chip2,
	Canyon,
}
