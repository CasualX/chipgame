use std::mem;
use std::path;
use std::fs;

use super::*;

mod event;
mod savedata;
mod lvsets;

pub use self::event::*;
pub use self::savedata::*;
pub use self::lvsets::*;

#[derive(Default)]
pub struct PlayState {
	pub fx: Option<fx::FxState>,
	pub menu: menu::MenuState,
	pub events: Vec<PlayEvent>,
	pub input: chipcore::Input,
	pub lvsets: LevelSets,
	pub save_data: SaveData,
}

impl PlayState {
	pub fn launch(&mut self, g: &mut shade::Graphics) {
		if self.lvsets.collection.is_empty() {
			return;
		}
		let mut splash = Vec::new();
		for set in &self.lvsets.collection {
			let Some(path) = &set.splash
			else {
				splash.push(None);
				continue
			};
			let props = shade::image::TextureProps {
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::ClampEdge,
				wrap_v: shade::TextureWrap::ClampEdge,
			};
			match shade::image::AnimatedImage::load(g, None, path, &props) {
				Ok(texs) => {
					eprintln!("Loaded splash image: {}", path.display());
					splash.push(Some(texs));
				}
				Err(err) => {
					eprintln!("Error loading splash image: {:?}", err);
					splash.push(None);
				}
			}
		}
		self.menu.stack.push(menu::Menu::PackSelect(menu::LevelPackSelectMenu {
			selected: self.lvsets.selected,
			items: self.lvsets.collection.iter().map(|lp| lp.title.clone()).collect(),
			splash,
			ntime: 0,
		}));
	}

	pub fn think(&mut self, input: &chipcore::Input) {
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
		let Some(lv_data) = self.lvsets.current().lv_data.get((level_number - 1) as usize) else { return };

		let attempts = self.save_data.update_level_attempts((level_number - 1) as usize);
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();
		self.save_data.current_level = level_number;
		self.save_data.save(&self.lvsets.current());

		fx.init();
		fx.parse_level(level_number, lv_data);
		fx.gs.ps.attempts = attempts;
		fx.camera.perspective = self.save_data.perspective;

		self.menu.close_all();
		self.events.push(PlayEvent::PlayLevel);
		self.play_music();
	}

	pub fn toggle_music(&mut self) {
		self.save_data.bg_music = !self.save_data.bg_music;
		self.save_data.save(&self.lvsets.current());
		self.play_music();
	}

	fn play_music(&mut self) {
		let music = if !self.save_data.bg_music {
			None
		}
		else if let Some(fx) = &self.fx {
			match fx.level_number % 2 {
				0 => Some(data::MusicId::Chip1),
				_ => Some(data::MusicId::Chip2),
			}
		}
		else {
			Some(data::MusicId::Canyon)
		};
		self.events.push(PlayEvent::PlayMusic { music });
	}

	pub fn sync(&mut self) {
		let events = mem::replace(&mut self.menu.events, Vec::new());
		for evt in events {
			eprintln!("MenuEvent: {:?}", evt);
			match evt {
				menu::MenuEvent::LevelPackSelect { index } => {
					self.lvsets.selected = index;
					self.save_data.load(&self.lvsets.current());
					self.save_data.save(&self.lvsets.current());
					self.menu.open_main(self.save_data.current_level > 0, &self.lvsets.current().title);
					self.play_music();
				}
				menu::MenuEvent::NewGame => {
					self.play_level(1);
				}
				menu::MenuEvent::MainMenu => {
					self.fx = None;
					self.events.push(PlayEvent::PlayLevel);
					self.menu.open_main(self.save_data.current_level > 0, &self.lvsets.current().title);
					self.play_music();
				}
				menu::MenuEvent::LevelPreview { level_number } => {
					for menu in &mut self.menu.stack {
						match menu {
							menu::Menu::LevelSelect(menu) => {
								if let Some(lv_data) = self.lvsets.current().lv_data.get((level_number - 1) as usize) {
									let mut fx = Box::new(crate::fx::FxState::default());
									fx.init();
									fx.parse_level(level_number, lv_data);
									fx.hud_enabled = false;
									menu.preview = Some(fx);
								}
								else {
									menu.preview = None;
								}
							}
							_ => {}
						}
					}
				}
				menu::MenuEvent::LevelSelect => {
					let mut menu = menu::LevelSelectMenu::default();
					menu.load_items(&self.lvsets.current(), &self.save_data);
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
					let mut success = false;
					for (index, lv_info) in self.lvsets.current().lv_info.iter().enumerate() {
						if let Some(lv_pass) = &lv_info.password {
							if lv_pass.as_bytes() == code.as_slice() {
								let level_number = index as i32 + 1;
								self.save_data.unlock_level(level_number);
								success = true;
							}
						}
					}
					if success {
						self.menu.open_main(true, &self.lvsets.current().title);
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
						let replay = fx.gs.save_replay(fx.gs_realtime);
						let record = serde_json::to_string_pretty(&replay).unwrap();
						if let Err(err) = std::fs::write(format!("save/{}/replay/level{}.attempt{}.json", self.lvsets.current().name, fx.level_number, fx.gs.ps.attempts), record) {
							eprintln!("Error saving replay: {}", err);
						}
					}
				}
				menu::MenuEvent::About => {
					if let Some(about) = &self.lvsets.current().about {
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
						perspective: self.save_data.perspective,
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
				menu::MenuEvent::SetBackgroundMusic { value } => {
					if self.save_data.bg_music != value {
						self.save_data.bg_music = value;
						self.save_data.save(&self.lvsets.current());
						self.play_music();
					}
				}
				menu::MenuEvent::SetSoundEffects { value } => {
					if self.save_data.sound_fx != value {
						self.save_data.sound_fx = value;
						self.save_data.save(&self.lvsets.current());
					}
				}
				menu::MenuEvent::SetDeveloperMode { value } => {
					if self.save_data.dev_mode != value {
						self.save_data.dev_mode = value;
						self.save_data.save(&self.lvsets.current());
					}
				}
				menu::MenuEvent::SetPerspective { value } => {
					if self.save_data.perspective != value {
						self.save_data.perspective = value;
						self.save_data.save(&self.lvsets.current());
						if let Some(fx) = &mut self.fx {
							fx.camera.perspective = value;
						}
					}
				}
				menu::MenuEvent::CursorMove => {
					self.events.push(PlayEvent::PlaySound { sound: chipcore::SoundFx::CursorMove });
				}
				menu::MenuEvent::CursorSelect => {
					self.events.push(PlayEvent::PlaySound { sound: chipcore::SoundFx::CursorSelect });
				}
				menu::MenuEvent::CloseMenu => {
					self.menu.close_menu(self.fx.is_some());
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
						let scores = savedata::Scores {
							ticks: fx.gs.time,
							steps: fx.gs.ps.steps,
							// attempts: fx.gs.ps.attempts,
						};
						self.save_data.complete_level(fx.level_number, scores);
						self.save_data.save(&self.lvsets.current());

						let menu = menu::GameWinMenu {
							selected: 0,
							level_number: fx.level_number,
							level_name: fx.gs.field.name.clone(),
							attempts: fx.gs.ps.attempts,
							time: fx.gs.time,
							steps: fx.gs.ps.steps,
							bonks: fx.gs.ps.bonks,
							time_high_score: false,
							steps_high_score: false,
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
		});

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
