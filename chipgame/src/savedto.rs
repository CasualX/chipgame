use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct SaveDto {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub current_level: Option<String>,
	pub unlocked_levels: Vec<String>,
	#[serde(default)]
	pub records: RecordsDto,
	pub options: OptionsDto,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct RecordsDto {
	#[serde(default)]
	pub mintime: HashMap<String, RecordDto>,
	#[serde(default)]
	pub minsteps: HashMap<String, RecordDto>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct RecordDto {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub date: Option<String>,
	pub ticks: i32,
	pub realtime: f32,
	pub steps: i32,
	pub bonks: i32,
	pub seed: String,
	pub replay: String,
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
