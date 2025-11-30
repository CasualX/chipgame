//! Gameplay module.

use std::{mem, path, fs};

use super::*;

mod event;
mod savedata;
mod lvsets;
pub(crate) mod tiles;

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
	// Tracks that the current `fx` holds a preview (not an active run).
	is_preview: bool,
}

impl PlayState {
	pub fn launch(&mut self, g: &mut shade::Graphics) {
		if self.lvsets.collection.is_empty() {
			return;
		}
		let mut splash = Vec::new();
		for level_set in &self.lvsets.collection {
			let Some(data) = &level_set.splash
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
			if let Ok(texs) = shade::image::AnimatedImage::load_memory(g, None, data, &props) {
				splash.push(Some(texs));
			}
		}
		self.menu.stack.push(menu::Menu::PackSelect(menu::PackSelectMenu {
			selected: self.lvsets.selected,
			items: self.lvsets.collection.iter().map(|level_set| level_set.title.clone()).collect(),
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
		let Some(level) = self.lvsets.current().levels.get((level_number - 1) as usize) else { return };

		self.is_preview = false;

		let attempts = self.save_data.update_level_attempts(level_number);
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();
		self.save_data.current_level = level_number;
		self.save_data.save(&self.lvsets.current());

		let seed = chipcore::RngSeed::System;
		// let set_name = &self.lvsets.current().name;
		// let seed = if let Ok(replay) = fs::read_to_string(format!("chipcore/tests/replays/{set_name}/level{level_number}.json")) {
		// 	let replay: chipty::ReplayDto = serde_json::from_str(&replay).unwrap();
		// 	fx.replay = Some(chipty::decode(&replay.replay));
		// 	let seed = u64::from_str_radix(&replay.seed, 16).unwrap();
		// 	chipcore::RngSeed::Manual(seed)
		// }
		// else {
		// 	chipcore::RngSeed::System
		// };

		fx.render.tiles = &tiles::TILES_PLAY;
		fx.parse_level(level_number, level, seed);
		fx.gs.ps.attempts = attempts;
		fx.camera.set_perspective(self.save_data.perspective);

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
		else if let Some(_) = &self.fx {
			if self.is_preview {
				Some(chipty::MusicId::MenuMusic)
			}
			else {
				Some(chipty::MusicId::GameMusic)
			}
		}
		else {
			Some(chipty::MusicId::MenuMusic)
		};
		self.events.push(PlayEvent::PlayMusic { music });
	}

	pub fn sync(&mut self) {
		let events = mem::replace(&mut self.menu.events, Vec::new());
		for evt in events {
			// eprintln!("MenuEvent: {:?}", evt);
			match evt {
				menu::MenuEvent::LoadLevelPack { index } => {
					self.lvsets.selected = index;
					self.save_data.load(&self.lvsets.current());
					self.save_data.save(&self.lvsets.current());
					self.menu.open_main(self.save_data.current_level > 0, &self.lvsets.current().title);
					self.play_music();
				}
				menu::MenuEvent::NewGame => {
					self.play_level(1);
				}
				menu::MenuEvent::OpenMainMenu => {
					self.fx = None;
					self.events.push(PlayEvent::PlayLevel);
					self.menu.open_main(self.save_data.current_level > 0, &self.lvsets.current().title);
					self.play_music();
				}
				menu::MenuEvent::PreviewLevel { level_number } => {
					// Reuse PlayState.fx to show a preview of the requested level
					self.load_preview_level(level_number);
				}
				menu::MenuEvent::OpenGoToLevel => {
					let mut menu = menu::GoToLevel::default();
					menu.load_items(&self.lvsets.current(), &self.save_data);
					// Start previewing at the current level
					self.load_preview_level(self.save_data.current_level);
					self.menu.stack.push(menu::Menu::GoToLevel(menu));
				}
				menu::MenuEvent::OpenUnlockLevel => {
					let menu = menu::UnlockLevelMenu {
						selected: 0,
						password: [None; 4],
					};
					self.menu.stack.push(menu::Menu::UnlockLevel(menu));
				}
				menu::MenuEvent::EnterPassword { code } => {
					// Secret code to soft-unlock all levels
					if code == *b"CHIP" {
						self.save_data.show_hidden_levels ^= true;
						self.menu.open_main(true, &self.lvsets.current().title);
						return;
					}
					let mut success = false;
					for (index, level) in self.lvsets.current().levels.iter().enumerate() {
						if let Some(level_password) = &level.password {
							if level_password.as_bytes() == code.as_slice() {
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
				menu::MenuEvent::PlayNextLevel => {
					let level_number = if let Some(fx) = &self.fx { fx.level_number + 1 } else { 1 };
					self.play_level(level_number);
				}
				menu::MenuEvent::RetryLevel | menu::MenuEvent::RestartLevel => {
					let level_number = if let Some(fx) = &self.fx { fx.level_number } else { 1 };
					self.play_level(level_number);
				}
				menu::MenuEvent::Continue => {
					let level_number = i32::max(1, self.save_data.current_level);
					self.play_level(level_number);
				}
				menu::MenuEvent::ResumePlay => {
					if let Some(fx) = &mut self.fx {
						self.menu.close_all();
						fx.unpause();
					}
				}
				menu::MenuEvent::SaveReplay => {
					if let Some(fx) = &self.fx {
						save_replay(self.lvsets.current(), fx);
					}
				}
				menu::MenuEvent::OpenAbout => {
					if let Some(about) = &self.lvsets.current().about {
						let menu = menu::AboutMenu {
							text: about.clone(),
						};
						self.menu.stack.push(menu::Menu::About(menu));
					}
				}
				menu::MenuEvent::ExitGame => {
					self.menu.close_all();
					self.fx = None;
					self.events.push(PlayEvent::Quit);
				}
				menu::MenuEvent::OpenOptions => {
					let menu = menu::OptionsMenu {
						selected: 0,
						bg_music: self.save_data.bg_music,
						sound_fx: self.save_data.sound_fx,
						dev_mode: self.save_data.dev_mode,
						perspective: self.save_data.perspective,
					};
					self.menu.stack.push(menu::Menu::Options(menu));
				}
				menu::MenuEvent::OpenPauseMenu => {
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
				menu::MenuEvent::OpenScoutMode => {
					if let Some(_) = &self.fx {
						let menu = menu::ScoutMode::default();
						self.menu.stack.push(menu::Menu::Scout(menu));
					}
				}
				menu::MenuEvent::ScoutN => {
					if let Some(fx) = &mut self.fx {
						let speed = if self.input.a { 5.0 } else { 2.0 };
						fx.scout_dir(chipty::Compass::Up, speed);
					}
				}
				menu::MenuEvent::ScoutE => {
					if let Some(fx) = &mut self.fx {
						let speed = if self.input.a { 5.0 } else { 2.0 };
						fx.scout_dir(chipty::Compass::Right, speed);
					}
				}
				menu::MenuEvent::ScoutS => {
					if let Some(fx) = &mut self.fx {
						let speed = if self.input.a { 5.0 } else { 2.0 };
						fx.scout_dir(chipty::Compass::Down, speed);
					}
				}
				menu::MenuEvent::ScoutW => {
					if let Some(fx) = &mut self.fx {
						let speed = if self.input.a { 5.0 } else { 2.0 };
						fx.scout_dir(chipty::Compass::Left, speed);
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
							fx.camera.set_perspective(value);
						}
					}
				}
				menu::MenuEvent::CursorMove => {
					self.events.push(PlayEvent::PlaySound { sound: chipty::SoundFx::CursorMove });
				}
				menu::MenuEvent::CursorSelect => {
					self.events.push(PlayEvent::PlaySound { sound: chipty::SoundFx::CursorSelect });
				}
				menu::MenuEvent::CloseMenu => {
					self.menu.close_menu(self.fx.is_some());
				}
				menu::MenuEvent::PreviewExit => {
					if self.is_preview && self.fx.is_some() {
						self.fx = None;
						self.is_preview = false;
					}
				}
			}
		}

		if let Some(fx) = &mut self.fx {
			let events = mem::replace(&mut fx.events, Vec::new());
			for evt in events {
				// eprintln!("FxEvent: {:?}", evt);
				match evt {
					fx::FxEvent::Sound(sound) => play_fx_play_sound(self, sound),
					fx::FxEvent::Scout => play_fx_scout(self),
					fx::FxEvent::Pause => play_fx_pause(self),
					fx::FxEvent::Unpause => play_fx_unpause(self),
					fx::FxEvent::LevelComplete => play_fx_level_complete(self),
					fx::FxEvent::GameOver => play_fx_game_over(self),
					// _ => {}
				}
			}
		}
	}

	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources, time: f64) {
		render::drawbg(g, resx);

		if let Some(fx) = &mut self.fx {
			fx.hud_enabled = self.menu.stack.is_empty();
			fx.draw(g, resx, time);
		}
		if self.fx.is_some() && !self.menu.stack.is_empty() {
			menu::darken(g, resx, 168);
		}
		self.menu.draw(g, resx, time);
	}

	fn load_preview_level(&mut self, level_number: i32) {
		// Only parse if the level exists in the current set
		if let Some(level) = self.lvsets.current().levels.get((level_number - 1) as usize) {
			let mut preview = fx::FxState::default();
			preview.render.tiles = &tiles::TILES_PLAY;
			preview.parse_level(level_number, level, chipcore::RngSeed::System);
			preview.camera.set_perspective(self.save_data.perspective);
			// HUD hidden when any menu is open; leave runtime flags at defaults
			self.fx = Some(preview);
			self.is_preview = true;
		}
		else {
			self.fx = None;
		}
	}
}

fn play_fx_play_sound(this: &mut PlayState, sound: chipty::SoundFx) {
	if this.save_data.sound_fx {
		this.events.push(PlayEvent::PlaySound { sound });
	}
}

fn play_fx_scout(this: &mut PlayState) {
	let Some(_fx) = &mut this.fx else {
		return
	};

	let menu = menu::ScoutMode::default();
	this.menu.stack.push(menu::Menu::Scout(menu));
}

fn play_fx_pause(this: &mut PlayState) {
	let Some(fx) = &this.fx else {
		return
	};

	let menu = menu::PauseMenu {
		selected: 0,
		level_number: fx.level_number,
		level_name: fx.gs.field.name.clone(),
		attempts: fx.gs.ps.attempts,
		time: fx.gs.time,
		steps: fx.gs.ps.steps,
		bonks: fx.gs.ps.bonks,
	};
	this.menu.stack.push(menu::Menu::Pause(menu));
}

fn play_fx_unpause(this: &mut PlayState) {
	this.menu.close_all();
}

fn play_fx_level_complete(this: &mut PlayState) {
	let Some(fx) = &this.fx else {
		return
	};

	let scores = savedata::Scores {
		ticks: fx.gs.time,
		steps: fx.gs.ps.steps,
	};
	let time_high_score = this.save_data.get_time_high_score(fx.level_number);
	let steps_high_score = this.save_data.get_steps_high_score(fx.level_number);
	let high_score =
		(time_high_score < 0 || scores.ticks < time_high_score) ||
		(steps_high_score < 0 || scores.steps < steps_high_score);
	if high_score {
		this.events.push(PlayEvent::PlaySound { sound: chipty::SoundFx::GameWin });
	}
	this.save_data.complete_level(fx.level_number, scores);
	this.save_data.save(&this.lvsets.current());
	// Auto-save replay if enabled or if a new high score was achieved
	if this.save_data.auto_save_replay || high_score {
		save_replay(this.lvsets.current(), fx);
	}

	let menu = menu::GameWinMenu {
		selected: 0,
		level_number: fx.level_number,
		level_name: fx.gs.field.name.clone(),
		attempts: fx.gs.ps.attempts,
		time: fx.gs.time,
		steps: fx.gs.ps.steps,
		bonks: fx.gs.ps.bonks,
		time_high_score,
		steps_high_score,
		..Default::default()
	};
	this.menu.stack.push(menu::Menu::GameWin(menu));
}

fn play_fx_game_over(this: &mut PlayState) {
	let Some(fx) = &this.fx else {
		return
	};

	let menu = menu::GameOverMenu {
		selected: 0,
		reason: fx.game_over,
		level_number: fx.level_number,
		level_name: fx.gs.field.name.clone(),
		attempts: fx.gs.ps.attempts,
		time: fx.gs.time,
		steps: fx.gs.ps.steps,
		bonks: fx.gs.ps.bonks,
	};
	this.menu.stack.push(menu::Menu::GameOver(menu));
}

fn save_replay(lvset: &LevelSet, fx: &fx::FxState) {
	let replay = fx.gs.save_replay(fx.game_realtime);
	let record = serde_json::to_string_pretty(&replay).unwrap();
	let path = format!("save/{}/replay/level{}.attempt{}.json", lvset.name, fx.level_number, fx.gs.ps.attempts);
	write_replay(path::Path::new(&path), &record);
}

fn write_replay(path: &path::Path, record: &str) {
	let _ = fs::create_dir(path.parent().unwrap_or(path));
	if let Err(err) = fs::write(path, record) {
		eprintln!("Error saving replay: {}", err);
	}
}
