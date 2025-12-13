use std::collections::{BTreeMap, BTreeSet};

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct SaveFileDto {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub current_level: Option<String>,
	pub unlocked_levels: BTreeSet<String>,
	#[serde(default)]
	pub completed_levels: BTreeSet<String>,
	#[serde(default)]
	pub high_scores: HighScoresDto,
	pub options: OptionsDto,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct HighScoresDto {
	#[serde(default)]
	pub ticks: BTreeMap<String, i32>,
	#[serde(default)]
	pub steps: BTreeMap<String, i32>,
	#[serde(default)]
	pub attempts: BTreeMap<String, i32>,
}

fn default_true() -> bool {
	true
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug)]
pub struct OptionsDto {
	pub background_music: bool,
	pub sound_effects: bool,
	pub developer_mode: bool,
	#[serde(default = "default_true")]
	pub perspective: bool,
	#[serde(default)]
	pub step_mode: bool,
	#[serde(default)]
	pub auto_save_replay: bool,
	#[serde(default)]
	pub speedrun_mode: bool,
}

impl Default for OptionsDto {
	fn default() -> OptionsDto {
		OptionsDto {
			background_music: true,
			sound_effects: true,
			developer_mode: false,
			perspective: true,
			step_mode: false,
			auto_save_replay: false,
			speedrun_mode: false,
		}
	}
}
