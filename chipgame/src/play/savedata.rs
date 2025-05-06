use std::path::Path;

use super::*;

pub struct HighScores {
	pub ticks: Vec<i32>,
	pub steps: Vec<i32>,
	pub attempts: Vec<i32>,
}

pub struct SaveData {
	pub current_level: i32,
	pub unlocked_levels: Vec<i32>,
	pub completed_levels: Vec<i32>,
	pub high_scores: HighScores,
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
}

impl Default for SaveData {
	fn default() -> Self {
		Self {
			current_level: 0,
			unlocked_levels: Vec::new(),
			completed_levels: Vec::new(),
			high_scores: HighScores {
				ticks: Vec::new(),
				steps: Vec::new(),
				attempts: Vec::new(),
			},
			bg_music: true,
			sound_fx: true,
			dev_mode: false,
		}
	}
}

impl SaveData {
	pub fn unlock_level(&mut self, level_number: i32) {
		if let Err(index) = self.unlocked_levels.binary_search(&level_number) {
			self.unlocked_levels.insert(index, level_number);
		}
		self.current_level = level_number;
	}
	pub fn complete_level(&mut self, level_number: i32) {
		if let Err(index) = self.completed_levels.binary_search(&level_number) {
			self.completed_levels.insert(index, level_number);
		}
		self.unlock_level(level_number + 1);
		self.current_level = level_number + 1;
	}

	pub fn is_level_unlocked(&self, level_number: i32) -> bool {
		self.unlocked_levels.binary_search(&level_number).is_ok()
	}

	pub fn save(&mut self, level_pack: &LevelSet, replay: Option<(i32, &chipcore::ReplayDto)>) {
		let file_name = get_levelset_state_filename(level_pack);

		let mut save_data = if let Ok(content) = std::fs::read_to_string(&file_name) {
			serde_json::from_str::<savedto::SaveDto>(&content).unwrap_or_default()
		}
		else {
			savedto::SaveDto::default()
		};

		if let Some((level_number, replay)) = replay {
			if let Some(level_name) = level_pack.lv_info.get(level_number as usize).map(|s| &s.name) {

				let level_ticks = save_data.high_scores.ticks.get(level_name).cloned().unwrap_or(i32::MAX);
				if replay.ticks < level_ticks {
					save_data.high_scores.ticks.insert(level_name.clone(), replay.ticks);
				}

				let level_steps = save_data.high_scores.steps.get(level_name).cloned().unwrap_or(i32::MAX);
				if replay.steps < level_steps {
					save_data.high_scores.steps.insert(level_name.clone(), replay.steps);
				}

				*save_data.high_scores.attempts.entry(level_name.clone()).or_default() += 1;
			}
		}

		let level_name = level_pack.lv_info.get((self.current_level - 1) as usize).map(|s| s.name.clone());
		save_data.current_level = level_name;


		save_data.options.background_music = self.bg_music;
		save_data.options.sound_effects = self.sound_fx;
		save_data.options.developer_mode = self.dev_mode;

		save_data.unlocked_levels.clear();
		let unlocked_levels = self.unlocked_levels.iter().filter_map(|&level_number| level_pack.lv_info.get((level_number - 1) as usize).map(|s| s.name.clone()));
		save_data.unlocked_levels.extend(unlocked_levels);

		let content = serde_json::to_string_pretty(&save_data).unwrap();
		// let content = encode_bytes(content.as_bytes());
		match std::fs::write(&file_name, content) {
			Ok(_) => {}
			Err(e) => eprintln!("Error saving file: {}", e),
		}
	}

	pub fn load(&mut self, level_pack: &LevelSet) {
		let file_name = get_levelset_state_filename(level_pack);

		let save_data = if let Ok(content) = std::fs::read_to_string(&file_name) {
			serde_json::from_str::<savedto::SaveDto>(&content).unwrap()
		}
		else {
			self.current_level = 0;
			self.unlocked_levels.clear();
			if level_pack.unlock_all_levels {
				self.unlocked_levels.extend(1..level_pack.lv_info.len() as i32 + 1);
			}
			else if self.unlocked_levels.is_empty() {
				self.unlocked_levels.push(1);
			}
			self.completed_levels.clear();
			self.high_scores.ticks.clear();
			self.high_scores.ticks.resize(level_pack.lv_info.len(), i32::MAX);
			self.high_scores.steps.clear();
			self.high_scores.steps.resize(level_pack.lv_info.len(), i32::MAX);
			self.high_scores.attempts.clear();
			self.high_scores.attempts.resize(level_pack.lv_info.len(), 0);
			return;
		};

		self.current_level = 0;
		if let Some(current_level) = save_data.current_level {
			if let Some(level_number) = level_pack.get_level_number(&current_level) {
				self.current_level = level_number;
			}
		}

		self.bg_music = save_data.options.background_music;
		self.sound_fx = save_data.options.sound_effects;
		self.dev_mode = save_data.options.developer_mode;

		self.unlocked_levels.clear();
		if level_pack.unlock_all_levels {
			self.unlocked_levels.extend(1..level_pack.lv_info.len() as i32 + 1);
		}
		else {
			let unlocked_levels = save_data.unlocked_levels.iter().filter_map(|level_name| level_pack.get_level_number(level_name));
			self.unlocked_levels.extend(unlocked_levels);
		}
		if self.unlocked_levels.is_empty() {
			self.unlocked_levels.push(1);
		}
		self.unlocked_levels.sort();
	}
}

fn get_levelset_state_filename(level_pack: &LevelSet) -> String {
	let filename = format!("save/{}/state.json", level_pack.name);
	let _ = std::fs::create_dir(Path::new(&filename).parent().unwrap());
	filename
}
