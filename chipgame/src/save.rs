use std::collections::HashMap;

pub struct SaveDto {
	pub records_time: HashMap<String, RecordDto>,
	pub records_steps: HashMap<String, RecordDto>,
	pub unlocked_levels: Vec<String>,
	pub options: OptionsDto,
}

pub struct RecordDto {
	pub date: String,
	pub frames: i64,
	pub steps: i32,
	pub bonks: i32,
	pub inputs: String,
}

pub struct OptionsDto {
	pub background_music: bool,
	pub sound_effects: bool,
	pub developer_mode: bool,
}
