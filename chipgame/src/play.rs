use std::mem;
use std::path;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<MusicId> },
	Quit,
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
	pub password: String,
}
#[derive(Default)]
pub struct LevelPack {
	pub name: String,
	pub title: String,
	pub lv_data: Vec<String>,
	pub lv_info: Vec<LevelData>,
}

#[derive(Default)]
pub struct PlayData {
	pub bg_music: bool,
	pub sound_fx: bool,
	pub dev_mode: bool,
	pub continue_level: i32,
	pub unlocked_levels: Vec<i32>,
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
			let s = std::fs::read_to_string(path.join(level)).unwrap();
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
	}

	pub fn launch(&mut self) {
		self.menu.open_main();
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

	pub fn play_level(&mut self, level_index: i32) {
		self.menu.close_all();
		let attempts = if let Some(fx) = &self.fx { if fx.level_index == level_index { fx.gs.ps.attempts } else { 0 } } else { 0 };
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();

		fx.init();
		fx.gs.ps.attempts = attempts;
		let lv_data = self.level_pack.lv_data[(level_index - 1) as usize].as_str();
		fx.parse_level(level_index, lv_data);
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
					self.menu.open_main();
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
				menu::MenuEvent::GoToLevel { level_index } => {
					self.play_level(level_index);
				}
				menu::MenuEvent::NextLevel => {
					let level_index = if let Some(fx) = &self.fx { fx.level_index + 1 } else { 1 };
					self.play_level(level_index);
				}
				menu::MenuEvent::Retry | menu::MenuEvent::Restart => {
					let level_index = if let Some(fx) = &self.fx { fx.level_index } else { 1 };
					self.play_level(level_index);
				}
				menu::MenuEvent::Continue => {
					let level_index = self.data.continue_level;
					self.play_level(level_index);
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
							level_index: fx.level_index,
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
					self.events.push(PlayEvent::PlayMusic { music: Some(MusicId::Canyon) });
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
							level_index: fx.level_index,
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
						let menu = menu::GameWinMenu {
							selected: 0,
							level_index: fx.level_index,
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
							level_index: fx.level_index,
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
