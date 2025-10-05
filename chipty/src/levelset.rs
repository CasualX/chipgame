use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum LevelRef {
	Indirect(String),
	Direct(LevelDto),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelSetDto {
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub about: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub splash: Option<String>,
	#[serde(default)]
	pub unlock_all_levels: bool,
	pub levels: Vec<LevelRef>,
}
