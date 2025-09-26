use std::collections::BTreeMap;
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
	pub unlocked_levels: Vec<bool>,
	pub completed_levels: Vec<bool>,
	pub high_scores: HighScores,
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
	pub perspective: bool,
	pub auto_save_replay: bool,
	pub show_hidden_levels: bool,
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
			auto_save_replay: false,
			show_hidden_levels: false,
		}
	}
}

impl SaveData {
	/// Unlocks the level and sets it as the current level.
	///
	/// When the previous level was completed or if the correct level password was entered.
	pub fn unlock_level(&mut self, level_number: i32) {
		let level_index = (level_number - 1) as usize;
		if let Some(lock) = self.unlocked_levels.get_mut(level_index) {
			*lock = true;
			self.current_level = level_number;
		}
	}

	/// Completes the level, unlocks the next level and saves the scores.
	pub fn complete_level(&mut self, level_number: i32, scores: Scores) {
		let level_index = (level_number - 1) as usize;
		if let Some(lock) = self.unlocked_levels.get_mut(level_index) {
			*lock = true;
		}

		self.unlock_level(level_number + 1);

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
		if self.show_hidden_levels {
			return true;
		}
		let level_index = (level_number - 1) as usize;
		self.unlocked_levels.get(level_index).copied().unwrap_or(false)
	}

	/// Updates and returns the current attempt for this level.
	pub fn update_level_attempts(&mut self, level_number: i32) -> i32 {
		let level_index = (level_number - 1) as usize;
		let Some(attempts_entry) = self.high_scores.attempts.get_mut(level_index)
		else {
			return 0;
		};
		*attempts_entry += 1;
		*attempts_entry
	}

	pub fn is_time_high_score(&self, level_number: i32, time: i32) -> bool {
		let level_index = (level_number - 1) as usize;
		let Some(ticks_entry) = self.high_scores.ticks.get(level_index) else { return false };
		return *ticks_entry < 0 || *ticks_entry > time;
	}

	pub fn is_steps_high_score(&self, level_number: i32, steps: i32) -> bool {
		let level_index = (level_number - 1) as usize;
		let Some(steps_entry) = self.high_scores.steps.get(level_index) else { return false };
		return *steps_entry < 0 || *steps_entry > steps;
	}

	pub fn save(&mut self, level_pack: &LevelSet) {
		let file_name = get_levelset_state_filename(level_pack);

		let level_name = level_pack.levels.get((self.current_level - 1) as usize).map(|lv| lv.field.name.clone());
		let unlocked_levels = (0..level_pack.levels.len()).filter_map(|level_index| {
			if !self.unlocked_levels[level_index] {
				return None;
			}
			Some(level_pack.levels[level_index].field.name.clone())
		});
		let completed_levels = (0..level_pack.levels.len()).filter_map(|level_index| {
			if !self.completed_levels[level_index] {
				return None;
			}
			Some(level_pack.levels[level_index].field.name.clone())
		});

		let save_data = SaveFileDto {
			current_level: level_name,
			unlocked_levels: unlocked_levels.collect(),
			completed_levels: completed_levels.collect(),
			high_scores: HighScoresDto {
				ticks: self.high_scores.ticks.iter().enumerate().filter(|(_, &v)| v >= 0).map(|(i, &v)| (level_pack.levels[i].field.name.clone(), v)).collect(),
				steps: self.high_scores.steps.iter().enumerate().filter(|(_, &v)| v >= 0).map(|(i, &v)| (level_pack.levels[i].field.name.clone(), v)).collect(),
				attempts: self.high_scores.attempts.iter().enumerate().filter(|(_, &v)| v > 0).map(|(i, &v)| (level_pack.levels[i].field.name.clone(), v)).collect(),
			},
			options: OptionsDto {
				background_music: self.bg_music,
				sound_effects: self.sound_fx,
				developer_mode: self.dev_mode,
				perspective: self.perspective,
				auto_save_replay: self.auto_save_replay,
			},
		};

		let content = serde_json::to_string_pretty(&save_data).unwrap();
		match std::fs::write(&file_name, content) {
			Ok(_) => {}
			Err(e) => eprintln!("Error saving {}: {}", file_name, e),
		}
	}

	pub fn load(&mut self, level_pack: &LevelSet) {
		*self = Self::load_refactored(level_pack);
	}

	fn load_refactored(level_pack: &LevelSet) -> SaveData {
		let file_name = get_levelset_state_filename(level_pack);

		let mut this = SaveData {
			current_level: 0,
			unlocked_levels: vec![level_pack.unlock_all_levels; level_pack.levels.len()],
			completed_levels: vec![false; level_pack.levels.len()],
			high_scores: HighScores {
				ticks: vec![-1; level_pack.levels.len()],
				steps: vec![-1; level_pack.levels.len()],
				attempts: vec![0; level_pack.levels.len()],
			},
			bg_music: true,
			sound_fx: true,
			dev_mode: false,
			perspective: true,
			auto_save_replay: true,
			show_hidden_levels: false,
		};
		this.unlocked_levels[0] = true;

		let Ok(content) = std::fs::read_to_string(&file_name) else {
			return this;
		};
		let Some(dto) = serde_json::from_str::<SaveFileDto>(&content).ok() else {
			return this;
		};

		if let Some(current_level) = &dto.current_level {
			if let Some(level_number) = level_pack.get_level_number(current_level) {
				this.current_level = level_number;
			}
		}

		this.bg_music = dto.options.background_music;
		this.sound_fx = dto.options.sound_effects;
		this.dev_mode = dto.options.developer_mode;
		this.perspective = dto.options.perspective;
		this.auto_save_replay = dto.options.auto_save_replay;

		for level_index in dto.unlocked_levels.iter().filter_map(|level_name| level_pack.get_level_index(level_name)) {
			this.unlocked_levels[level_index] = true;
		}

		for level_index in dto.completed_levels.iter().filter_map(|level_name| level_pack.get_level_index(level_name)) {
			this.completed_levels[level_index] = true;
		}

		fn load_high_scores(
			level_pack: &LevelSet,
			saved_data: &BTreeMap<String, i32>,
			scores: &mut Vec<i32>,
		) {
			for (level_name, score) in saved_data {
				if let Some(level_index) = level_pack.get_level_index(level_name) {
					scores[level_index] = *score;
				}
			}
		}
		load_high_scores(level_pack, &dto.high_scores.ticks, &mut this.high_scores.ticks);
		load_high_scores(level_pack, &dto.high_scores.steps, &mut this.high_scores.steps);
		load_high_scores(level_pack, &dto.high_scores.attempts, &mut this.high_scores.attempts);

		return this;
	}
}

fn get_levelset_state_filename(level_pack: &LevelSet) -> String {
	let filename = format!("save/{}/state.json", level_pack.name);
	let _ = std::fs::create_dir(Path::new(&filename).parent().unwrap());
	filename
}

pub use chipty::{SaveFileDto, HighScoresDto, OptionsDto};
