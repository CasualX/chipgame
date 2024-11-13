use std::mem;
use std::path;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<data::MusicId> },
	Quit,
	PlayLevel,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelPackDto {
	pub name: String,
	pub title: String,
	pub levels: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelData {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hint: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub password: Option<String>,
}
#[derive(Default)]
pub struct LevelPack {
	pub name: String,
	pub title: String,
	pub lv_data: Vec<String>,
	pub lv_info: Vec<LevelData>,
}
impl LevelPack {
	pub fn get_level_number(&self, name: &str) -> Option<i32> {
		self.lv_info.iter().position(|s| s.name == name).map(|i| i as i32 + 1)
	}
}

pub struct PlayData {
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
	pub current_level: i32,
	pub unlocked_levels: Vec<i32>,
}
impl Default for PlayData {
	fn default() -> Self {
		Self {
			bg_music: true,
			sound_fx: true,
			dev_mode: false,
			current_level: 0,
			unlocked_levels: Vec::new(),
		}
	}
}
impl PlayData {
	pub fn unlock_level(&mut self, level_number: i32) {
		if let Err(index) = self.unlocked_levels.binary_search(&level_number) {
			self.unlocked_levels.insert(index, level_number);
		}
	}
	pub fn is_level_unlocked(&self, level_number: i32) -> bool {
		self.unlocked_levels.binary_search(&level_number).is_ok()
	}
	pub fn load(&mut self, level_pack: &LevelPack) {
		let file_name = format!("save/{}.json", level_pack.name);

		let save_data = if let Ok(content) = std::fs::read_to_string(&file_name) {
			serde_json::from_str::<save::SaveDto>(&content).unwrap_or_default()
		}
		else {
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
		let unlocked_levels = save_data.unlocked_levels.iter().filter_map(|level_name| level_pack.get_level_number(level_name));
		self.unlocked_levels.extend(unlocked_levels);
		if self.unlocked_levels.is_empty() {
			self.unlocked_levels.push(1);
		}
		self.unlocked_levels.sort();
	}
	pub fn save(&mut self, level_pack: &LevelPack, replay: Option<(i32, save::RecordDto)>) {
		let file_name = format!("save/{}.json", level_pack.name);

		let mut save_data = if let Ok(content) = std::fs::read_to_string(&file_name) {
			serde_json::from_str::<save::SaveDto>(&content).unwrap_or_default()
		}
		else {
			save::SaveDto::default()
		};

		let level_name = level_pack.lv_info.get((self.current_level - 1) as usize).map(|s| s.name.clone());
		if let Some(level_name) = &level_name {
			if let Some((level_number, replay)) = replay {

				if let Some(entry) = save_data.records_time.get(level_name) {
					if replay.ticks < entry.ticks {
						save_data.records_time.insert(level_name.clone(), replay.clone());
					}
				}
				else {
					save_data.records_time.insert(level_name.clone(), replay.clone());
				}

				if let Some(entry) = save_data.records_steps.get(level_name) {
					if replay.steps < entry.steps || (replay.steps == entry.steps && replay.ticks < entry.ticks) {
						save_data.records_steps.insert(level_name.clone(), replay.clone());
					}
				}
				else {
					save_data.records_steps.insert(level_name.clone(), replay.clone());
				}
			}
		}
		save_data.current_level = level_name;


		save_data.options.background_music = self.bg_music;
		save_data.options.sound_effects = self.sound_fx;
		save_data.options.developer_mode = self.dev_mode;

		save_data.unlocked_levels.clear();
		let unlocked_levels = self.unlocked_levels.iter().filter_map(|&level_number| level_pack.lv_info.get((level_number - 1) as usize).map(|s| s.name.clone()));
		save_data.unlocked_levels.extend(unlocked_levels);

		let content = serde_json::to_string_pretty(&save_data).unwrap();
		match std::fs::write(&file_name, content) {
			Ok(_) => {}
			Err(e) => eprintln!("Error saving file: {}", e),
		}
	}
}

#[derive(Default)]
pub struct PlayState {
	pub fx: Option<fx::FxState>,
	pub menu: menu::MenuState,
	pub events: Vec<PlayEvent>,
	pub input: core::Input,
	pub level_pack: LevelPack,
	pub data: PlayData,
}

impl PlayState {
	pub fn load_pack(&mut self, path: &path::Path) {
		let json = std::fs::read_to_string(path.join("index.json")).unwrap();
		let pack: LevelPackDto = serde_json::from_str(&json).unwrap();
		let mut lv_info = Vec::new();
		let mut lv_data = Vec::new();
		for level in &pack.levels {
			let s = std::fs::read_to_string(path.join(level)).expect(level);
			let ld: LevelData = serde_json::from_str(&s).unwrap();
			lv_info.push(ld);
			lv_data.push(s);
		}
		self.level_pack = LevelPack {
			name: pack.name,
			title: pack.title,
			lv_data,
			lv_info,
		};
		self.data.load(&self.level_pack);
		self.data.save(&self.level_pack, None);
	}

	pub fn launch(&mut self) {
		self.menu.open_main(self.data.current_level > 0);
	}

	pub fn think(&mut self, input: &core::Input) {
		{
			let input = menu::Input {
				a: menu::KeyState::w(self.input.a, input.a),
				b: menu::KeyState::w(self.input.b, input.b),
				up: menu::KeyState::w(self.input.up, input.up),
				down: menu::KeyState::w(self.input.down, input.down),
				left: menu::KeyState::w(self.input.left, input.left),
				right: menu::KeyState::w(self.input.right, input.right),
				start: menu::KeyState::w(self.input.start, input.start),
				select: menu::KeyState::w(self.input.select, input.select),
			};
			self.menu.think(&input);
			if self.menu.stack.is_empty() {
				if let Some(fx) = &mut self.fx {
					fx.think(&input);
				}
			}
		}
		self.input = *input;

		self.sync();
	}

	pub fn play_level(&mut self, level_number: i32) {
		// If loading a level fails just... do nothing
		let Some(lv_data) = self.level_pack.lv_data.get((level_number - 1) as usize) else { return };

		let attempts = if let Some(fx) = &self.fx { if fx.level_number == level_number { fx.gs.ps.attempts } else { 0 } } else { 0 };
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();
		self.data.current_level = level_number;

		fx.init();
		fx.gs.ps.attempts = attempts;
		fx.parse_level(level_number, lv_data);

		self.menu.close_all();
		self.events.push(PlayEvent::PlayLevel);
	}

	pub fn sync(&mut self) {
		let events = mem::replace(&mut self.menu.events, Vec::new());
		for evt in events {
			println!("MenuEvent: {:?}", evt);
			match evt {
				menu::MenuEvent::NewGame => {
					self.play_level(1);
				}
				menu::MenuEvent::MainMenu => {
					self.fx = None;
					self.events.push(PlayEvent::PlayLevel);
					self.menu.open_main(self.data.current_level > 0);
				}
				menu::MenuEvent::LevelSelect => {
					let mut menu = menu::LevelSelectMenu {
						selected: 0,
						offset: 0,
						items: Vec::new(),
					};
					menu.load_items(&self.level_pack);
					self.menu.stack.push(menu::Menu::LevelSelect(menu));
				}
				menu::MenuEvent::PlayLevel { level_number } => {
					self.play_level(level_number);
				}
				menu::MenuEvent::NextLevel => {
					let level_number = if let Some(fx) = &self.fx { fx.level_number + 1 } else { 1 };
					self.play_level(level_number);
				}
				menu::MenuEvent::Retry | menu::MenuEvent::Restart => {
					let level_number = if let Some(fx) = &self.fx { fx.level_number } else { 1 };
					self.play_level(level_number);
				}
				menu::MenuEvent::Continue => {
					let level_number = i32::max(1, self.data.current_level);
					self.play_level(level_number);
				}
				menu::MenuEvent::Resume => {
					if let Some(fx) = &mut self.fx {
						self.menu.close_all();
						fx.unpause();
					}
				}
				menu::MenuEvent::Exit => {
					self.menu.close_all();
					self.fx = None;
					self.events.push(PlayEvent::Quit);
				}
				menu::MenuEvent::Options => {
					let menu = menu::OptionsMenu {
						selected: 0,
						bg_music: self.data.bg_music,
						sound_fx: self.data.sound_fx,
						dev_mode: self.data.dev_mode,
					};
					self.menu.stack.push(menu::Menu::Options(menu));
				}
				menu::MenuEvent::PauseMenu => {
					if let Some(fx) = &mut self.fx {
						let menu = menu::PauseMenu {
							selected: 0,
							level_number: fx.level_number,
							level_name: fx.gs.field.name.clone(),
							attempts: fx.gs.ps.attempts,
							time: fx.gs.time,
							steps: fx.gs.ps.steps,
							bonks: fx.gs.ps.bonks,
						};
						self.menu.stack.push(menu::Menu::Pause(menu));
					}
				}
				menu::MenuEvent::BgMusicOn => {
					self.data.bg_music = true;
					self.events.push(PlayEvent::PlayMusic { music: Some(data::MusicId::Canyon) });
				}
				menu::MenuEvent::BgMusicOff => {
					self.data.bg_music = false;
					self.events.push(PlayEvent::PlayMusic { music: None });
				}
				menu::MenuEvent::SoundFxOn => {
					self.data.sound_fx = true;
				}
				menu::MenuEvent::SoundFxOff => {
					self.data.sound_fx = false;
				}
				menu::MenuEvent::DevModeOn => {
					self.data.dev_mode = true;
				}
				menu::MenuEvent::DevModeOff => {
					self.data.dev_mode = false;
				}
				menu::MenuEvent::CursorMove => {}
				menu::MenuEvent::CloseMenu => {
					self.menu.close_menu();
				}
				_ => unimplemented!("{:?}", evt),
			}
		}

		if let Some(fx) = &mut self.fx {
			let events = mem::replace(&mut fx.events, Vec::new());
			for evt in events {
				println!("FxEvent: {:?}", evt);
				match evt {
					fx::FxEvent::PlaySound { sound } => {
						self.events.push(PlayEvent::PlaySound { sound });
					}
					fx::FxEvent::PlayMusic { music } => {
						self.events.push(PlayEvent::PlayMusic { music });
					}
					fx::FxEvent::Pause => {
						let menu = menu::PauseMenu {
							selected: 0,
							level_number: fx.level_number,
							level_name: fx.gs.field.name.clone(),
							attempts: fx.gs.ps.attempts,
							time: fx.gs.time,
							steps: fx.gs.ps.steps,
							bonks: fx.gs.ps.bonks,
						};
						self.menu.stack.push(menu::Menu::Pause(menu));
					}
					fx::FxEvent::Unpause => {
						self.menu.close_all();
					}
					fx::FxEvent::GameWin => {
						self.data.unlock_level(fx.level_number);
						self.data.unlock_level(fx.level_number + 1);
						self.data.current_level = fx.level_number + 1;
						self.data.save(&self.level_pack, Some((fx.level_number, get_record_data_from_fx(fx))));
						let menu = menu::GameWinMenu {
							selected: 0,
							level_number: fx.level_number,
							level_name: fx.gs.field.name.clone(),
							attempts: fx.gs.ps.attempts,
							time: fx.gs.time,
							steps: fx.gs.ps.steps,
							bonks: fx.gs.ps.bonks,
						};
						self.menu.stack.push(menu::Menu::GameWin(menu));
					}
					fx::FxEvent::GameOver => {
						let menu = menu::GameOverMenu {
							selected: 0,
							activity: fx.gs.ps.activity,
							level_number: fx.level_number,
							level_name: fx.gs.field.name.clone(),
							attempts: fx.gs.ps.attempts,
							time: fx.gs.time,
							steps: fx.gs.ps.steps,
							bonks: fx.gs.ps.bonks,
						};
						self.menu.stack.push(menu::Menu::GameOver(menu));
					}
					// _ => {}
				}
			}
		}
	}

	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources) {
		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(cvmath::Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		}).unwrap();

		if let Some(fx) = &mut self.fx {
			fx.hud_enabled = self.menu.stack.is_empty();
			fx.draw(g, resx);
		}
		if self.fx.is_some() && !self.menu.stack.is_empty() {
			menu::darken(g, resx, 168);
		}
		self.menu.draw(g, resx);
	}

}

fn get_record_data_from_fx(fx: &fx::FxState) -> save::RecordDto {
	let mut replay = String::new();
	for input in &fx.gs.inputs {
		use std::fmt::Write;
		let _ = write!(replay, "{:02x}", input);
	}

	save::RecordDto {
		date: None,
		ticks: fx.gs.time,
		steps: fx.gs.ps.steps,
		bonks: fx.gs.ps.bonks,
		replay,
	}
}
