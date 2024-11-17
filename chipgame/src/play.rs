use std::mem;
use std::path;
use std::fs;

use super::*;

mod event;
mod savedata;

pub use self::event::*;
pub use self::savedata::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelPackDto {
	pub name: String,
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub about: Option<Vec<String>>,
	#[serde(default)]
	pub unlock_all_levels: bool,
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
	pub about: Option<String>,
	pub unlock_all_levels: bool,
	pub lv_data: Vec<String>,
	pub lv_info: Vec<LevelData>,
}
impl LevelPack {
	pub fn get_level_number(&self, name: &str) -> Option<i32> {
		self.lv_info.iter().position(|s| s.name == name).map(|i| i as i32 + 1)
	}
}

#[derive(Default)]
pub struct PlayState {
	pub fx: Option<fx::FxState>,
	pub menu: menu::MenuState,
	pub events: Vec<PlayEvent>,
	pub input: core::Input,
	pub lp_index: usize,
	pub level_packs: Vec<LevelPack>,
	pub save_data: SaveData,
}

fn load_level_packs(packs: &mut Vec<LevelPack>) {
	let dir = match fs::read_dir("data/packs") {
		Ok(dir) => dir,
		Err(err) => {
			eprintln!("Error reading data/packs directory: {}", err);
			return;
		}
	};
	for entry in dir {
		match entry {
			Ok(entry) => {
				let path = entry.path();
				if path.is_dir() {
					load_level_pack(&path, packs);
				}
			}
			Err(err) => {
				eprintln!("Error reading pack: {}", err);
			}
		}
	}
	packs.sort_by_key(|lp| lp.name.clone());
}

fn load_level_pack(path: &path::Path, packs: &mut Vec<LevelPack>) {
	let index_path = path.join("index.json");
	let json = match fs::read_to_string(&index_path) {
		Ok(json) => json,
		Err(err) => {
			eprintln!("Error reading {}: {}", index_path.display(), err);
			return;
		}
	};

	let pack: LevelPackDto = match serde_json::from_str(&json) {
		Ok(pack) => pack,
		Err(err) => {
			eprintln!("Error parsing {}: {}", index_path.display(), err);
			return;
		}
	};

	let mut lv_info = Vec::new();
	let mut lv_data = Vec::new();
	for level in &pack.levels {
		let level_path = path.join(level);
		let s = match fs::read_to_string(&level_path) {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Error reading {}: {}", level_path.display(), err);
				continue;
			}
		};

		let ld: LevelData = match serde_json::from_str(&s) {
			Ok(ld) => ld,
			Err(err) => {
				eprintln!("Error parsing {}: {}", level_path.display(), err);
				continue;
			}
		};

		lv_info.push(ld);
		lv_data.push(s);
	}

	packs.push(LevelPack {
		name: pack.name,
		title: pack.title,
		about: pack.about.map(|lines| lines.join("\n")),
		unlock_all_levels: pack.unlock_all_levels,
		lv_data,
		lv_info,
	});
}

impl PlayState {
	pub fn load_packs(&mut self) {
		load_level_packs(&mut self.level_packs);
	}

	pub fn launch(&mut self) {
		if self.level_packs.is_empty() {
			return;
		}
		self.menu.stack.push(menu::Menu::PackSelect(menu::LevelPackSelectMenu {
			selected: 0,
			items: self.level_packs.iter().map(|lp| lp.title.clone()).collect(),
		}));
		// self.menu.open_main(self.save_data.current_level > 0, &self.level_packs[self.lp_index].title);
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
		let Some(lv_data) = self.level_packs[self.lp_index].lv_data.get((level_number - 1) as usize) else { return };

		let attempts = if let Some(fx) = &self.fx { if fx.level_number == level_number { fx.gs.ps.attempts } else { 0 } } else { 0 };
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();
		self.save_data.current_level = level_number;
		self.save_data.save(&self.level_packs[self.lp_index], None);

		fx.init();
		fx.gs.ps.attempts = attempts;
		fx.parse_level(level_number, lv_data);

		self.menu.close_all();
		self.events.push(PlayEvent::PlayLevel);
	}

	pub fn sync(&mut self) {
		let events = mem::replace(&mut self.menu.events, Vec::new());
		for evt in events {
			eprintln!("MenuEvent: {:?}", evt);
			match evt {
				menu::MenuEvent::LevelPackSelect { index } => {
					self.lp_index = index;
					self.save_data.load(&self.level_packs[self.lp_index]);
					self.save_data.save(&self.level_packs[self.lp_index], None);
					self.menu.open_main(self.save_data.current_level > 0, &self.level_packs[self.lp_index].title);
				}
				menu::MenuEvent::NewGame => {
					self.play_level(1);
				}
				menu::MenuEvent::MainMenu => {
					self.fx = None;
					self.events.push(PlayEvent::PlayLevel);
					self.menu.open_main(self.save_data.current_level > 0, &self.level_packs[self.lp_index].title);
				}
				menu::MenuEvent::LevelSelect => {
					let mut menu = menu::LevelSelectMenu {
						selected: 0,
						offset: 0,
						items: Vec::new(),
					};
					menu.load_items(&self.level_packs[self.lp_index], &self.save_data);
					self.menu.stack.push(menu::Menu::LevelSelect(menu));
				}
				menu::MenuEvent::UnlockLevel => {
					let menu = menu::UnlockLevelMenu {
						selected: 0,
						password: [None; 4],
					};
					self.menu.stack.push(menu::Menu::UnlockLevel(menu));
				}
				menu::MenuEvent::EnterPassword { code } => {
					for (index, lv_info) in self.level_packs[self.lp_index].lv_info.iter().enumerate() {
						if let Some(lv_pass) = &lv_info.password {
							if lv_pass.as_bytes() == code.as_slice() {
								let level_number = index as i32 + 1;
								self.save_data.unlock_level(level_number);
								self.save_data.current_level = level_number;
							}
						}
					}
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
					let level_number = i32::max(1, self.save_data.current_level);
					self.play_level(level_number);
				}
				menu::MenuEvent::Resume => {
					if let Some(fx) = &mut self.fx {
						self.menu.close_all();
						fx.unpause();
					}
				}
				menu::MenuEvent::SaveReplay => {
					if let Some(fx) = &self.fx {
						let record = get_record_data_from_fx(fx);
						let record = serde_json::to_string_pretty(&record).unwrap();
						if let Err(err) = std::fs::write(format!("replay/{}.level{}.attempt{}.json", self.level_packs[self.lp_index].name, fx.level_number, fx.gs.ps.attempts), record) {
							eprintln!("Error saving replay: {}", err);
						}
					}
				}
				menu::MenuEvent::About => {
					if let Some(about) = &self.level_packs[self.lp_index].about {
						let menu = menu::AboutMenu {
							text: about.clone(),
						};
						self.menu.stack.push(menu::Menu::About(menu));
					}
				}
				menu::MenuEvent::HighScores => {}
				menu::MenuEvent::Exit => {
					self.menu.close_all();
					self.fx = None;
					self.events.push(PlayEvent::Quit);
				}
				menu::MenuEvent::Options => {
					let menu = menu::OptionsMenu {
						selected: 0,
						bg_music: self.save_data.bg_music,
						sound_fx: self.save_data.sound_fx,
						dev_mode: self.save_data.dev_mode,
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
					self.save_data.bg_music = true;
					self.events.push(PlayEvent::PlayMusic { music: Some(data::MusicId::Canyon) });
				}
				menu::MenuEvent::BgMusicOff => {
					self.save_data.bg_music = false;
					self.events.push(PlayEvent::PlayMusic { music: None });
				}
				menu::MenuEvent::SoundFxOn => {
					self.save_data.sound_fx = true;
				}
				menu::MenuEvent::SoundFxOff => {
					self.save_data.sound_fx = false;
				}
				menu::MenuEvent::DevModeOn => {
					self.save_data.dev_mode = true;
				}
				menu::MenuEvent::DevModeOff => {
					self.save_data.dev_mode = false;
				}
				menu::MenuEvent::CursorMove => {}
				menu::MenuEvent::CloseMenu => {
					self.menu.close_menu();
				}
			}
		}

		if let Some(fx) = &mut self.fx {
			let events = mem::replace(&mut fx.events, Vec::new());
			for evt in events {
				eprintln!("FxEvent: {:?}", evt);
				match evt {
					fx::FxEvent::PlaySound { sound } => {
						if self.save_data.sound_fx {
							self.events.push(PlayEvent::PlaySound { sound });
						}
					}
					fx::FxEvent::PlayMusic { mut music } => {
						if !self.save_data.bg_music {
							music = None;
						}
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
						self.save_data.unlock_level(fx.level_number);
						self.save_data.unlock_level(fx.level_number + 1);
						self.save_data.current_level = fx.level_number + 1;
						self.save_data.save(&self.level_packs[self.lp_index], Some((fx.level_number, &get_record_data_from_fx(fx))));

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

fn encode_bytes(bytes: &[u8]) -> String {
	// Compress the bytes
	let mut compressed = Vec::new();
	compressed.reserve(bytes.len());
	let mut compress = flate2::Compress::new(flate2::Compression::best(), true);
	compress.compress_vec(bytes, &mut compressed, flate2::FlushCompress::Finish).unwrap();

	// Base64 encode to string
	simple_base64::encode_engine(compressed.as_slice(), &simple_base64::engine::general_purpose::STANDARD_NO_PAD)
}

fn get_record_data_from_fx(fx: &fx::FxState) -> save::RecordDto {
	let replay = encode_bytes(&fx.gs.inputs);

	save::RecordDto {
		date: None,
		ticks: fx.gs.time,
		realtime: fx.gs_realtime,
		steps: fx.gs.ps.steps,
		bonks: fx.gs.ps.bonks,
		seed: format!("{:016x}", fx.gs.field.seed),
		replay,
	}
}
