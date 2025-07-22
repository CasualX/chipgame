use std::collections::HashMap;
use std::path::Path;

use super::*;

pub struct Scores {
	pub ticks: i32,
	pub steps: i32,
	// pub attempts: i32,
}

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
	pub perspective: bool,
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
			perspective: true,
		}
	}
}

impl SaveData {
	/// Unlocks the level and sets it as the current level.
	///
	/// When the previous level was completed or if the correct level password was entered.
	pub fn unlock_level(&mut self, level_number: i32) {
		if let Err(index) = self.unlocked_levels.binary_search(&level_number) {
			self.unlocked_levels.insert(index, level_number);
		}
		self.current_level = level_number;
	}
	pub fn complete_level(&mut self, level_number: i32, scores: Scores) {
		if let Err(index) = self.completed_levels.binary_search(&level_number) {
			self.completed_levels.insert(index, level_number);
		}
		self.unlock_level(level_number + 1);
		self.current_level = level_number + 1;

		let level_index = (level_number - 1) as usize;
		if let Some(ticks_entry) = self.high_scores.ticks.get_mut(level_index) {
			if *ticks_entry < 0 || *ticks_entry > scores.ticks {
				*ticks_entry = scores.ticks;
			}
		}
		if let Some(steps_entry) = self.high_scores.steps.get_mut(level_index) {
			if *steps_entry < 0 || *steps_entry > scores.steps {
				*steps_entry = scores.steps;
			}
		}
		// if let Some(attempts_entry) = self.high_scores.attempts.get_mut(level_index) {
		// 	*attempts_entry = scores.attempts;
		// }
	}

	pub fn is_level_unlocked(&self, level_number: i32) -> bool {
		self.unlocked_levels.binary_search(&level_number).is_ok()
	}

	/// Updates and returns the current attempt for this level.
	pub fn update_level_attempts(&mut self, level_index: usize) -> i32 {
		let Some(attempts_entry) = self.high_scores.attempts.get_mut(level_index)
		else {
			return 0;
		};
		*attempts_entry += 1;
		*attempts_entry
	}

	pub fn save(&mut self, level_pack: &LevelSet) {
		let file_name = get_levelset_state_filename(level_pack);

		let level_name = level_pack.lv_info.get((self.current_level - 1) as usize).map(|s| s.name.clone());
		let unlocked_levels = self.unlocked_levels.iter().filter_map(|&level_number| level_pack.lv_info.get((level_number - 1) as usize).map(|s| s.name.clone()));
		let completed_levels = self.completed_levels.iter().filter_map(|&level_number| level_pack.lv_info.get((level_number - 1) as usize).map(|s| s.name.clone()));

		let mut save_data = SaveDto {
			current_level: level_name,
			unlocked_levels: unlocked_levels.collect(),
			completed_levels: completed_levels.collect(),
			high_scores: HighScoresDto {
				ticks: self.high_scores.ticks.iter().enumerate().filter(|(_, &v)| v >= 0).map(|(i, &v)| (level_pack.lv_info[i].name.clone(), v)).collect(),
				steps: self.high_scores.steps.iter().enumerate().filter(|(_, &v)| v >= 0).map(|(i, &v)| (level_pack.lv_info[i].name.clone(), v)).collect(),
				attempts: self.high_scores.attempts.iter().enumerate().filter(|(_, &v)| v > 0).map(|(i, &v)| (level_pack.lv_info[i].name.clone(), v)).collect(),
			},
			options: OptionsDto {
				background_music: self.bg_music,
				sound_effects: self.sound_fx,
				developer_mode: self.dev_mode,
				perspective: self.perspective,
			},
		};

		save_data.unlocked_levels.sort();
		save_data.completed_levels.sort();

		let content = serde_json::to_string_pretty(&save_data).unwrap();
		match std::fs::write(&file_name, content) {
			Ok(_) => {}
			Err(e) => eprintln!("Error saving file: {}", e),
		}
	}

	pub fn load(&mut self, level_pack: &LevelSet) {
		let file_name = get_levelset_state_filename(level_pack);

		let save_data = if let Ok(content) = std::fs::read_to_string(&file_name) {
			serde_json::from_str::<SaveDto>(&content).unwrap()
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
			self.high_scores.ticks.resize(level_pack.lv_info.len(), -1);
			self.high_scores.steps.clear();
			self.high_scores.steps.resize(level_pack.lv_info.len(), -1);
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
		self.perspective = save_data.options.perspective;

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
		self.unlocked_levels.dedup();

		self.completed_levels = save_data.completed_levels.iter().filter_map(|level_name| level_pack.get_level_number(level_name)).collect();
		self.completed_levels.sort();
		self.completed_levels.dedup();

		// Load high scores
		fn load_high_scores(
			level_pack: &LevelSet,
			saved_data: &HashMap<String, i32>,
			default_value: i32,
			scores: &mut Vec<i32>,
		) {
			scores.clear();
			scores.resize(level_pack.lv_info.len(), default_value);
			for (level_name, score) in saved_data {
				if let Some(level_index) = level_pack.get_level_index(level_name) {
					scores[level_index] = *score;
				}
			}
		}
		load_high_scores(level_pack, &save_data.high_scores.ticks, -1, &mut self.high_scores.ticks);
		load_high_scores(level_pack, &save_data.high_scores.steps, -1, &mut self.high_scores.steps);
		load_high_scores(level_pack, &save_data.high_scores.attempts, 0, &mut self.high_scores.attempts);
	}
}

fn get_levelset_state_filename(level_pack: &LevelSet) -> String {
	let filename = format!("save/{}/state.json", level_pack.name);
	let _ = std::fs::create_dir(Path::new(&filename).parent().unwrap());
	filename
}

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
}

impl Default for OptionsDto {
	fn default() -> Self {
		Self {
			background_music: true,
			sound_effects: true,
			developer_mode: false,
			perspective: true,
		}
	}
}
