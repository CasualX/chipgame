use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelSetIndirectDto {
	pub name: String,
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub about: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub splash: Option<String>,
	#[serde(default)]
	pub unlock_all_levels: bool,
	pub levels: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelSetDirectDto {
	pub name: String,
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub about: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub splash: Option<String>,
	#[serde(default)]
	pub unlock_all_levels: bool,
	pub levels: Vec<LevelDto>,
}
