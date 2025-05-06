use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct SaveDto {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub current_level: Option<String>,
	pub unlocked_levels: Vec<String>,
	#[serde(default)]
	pub completed_levels: Vec<String>,
	#[serde(default)]
	pub high_scores: HighScoresDto,
	pub options: OptionsDto,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct HighScoresDto {
	#[serde(default)]
	pub ticks: HashMap<String, i32>,
	#[serde(default)]
	pub steps: HashMap<String, i32>,
	#[serde(default)]
	pub attempts: HashMap<String, i32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug)]
pub struct OptionsDto {
	pub background_music: bool,
	pub sound_effects: bool,
	pub developer_mode: bool,
}

impl Default for OptionsDto {
	fn default() -> Self {
		Self {
			background_music: true,
			sound_effects: true,
			developer_mode: false,
		}
	}
}
