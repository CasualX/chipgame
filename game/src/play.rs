use std::mem;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<MusicId> },
}

#[derive(Default)]
pub struct PlayState {
	pub fx: Option<fx::FxState>,
	pub menu: menu::MenuState,
	pub events: Vec<PlayEvent>,
	pub input: core::Input,
}

impl PlayState {
	pub fn launch(&mut self) {
		self.menu.open_main();
	}

	pub fn load_level(&mut self, level_index: i32) {
		self.menu.close_all();
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();

		fx.init();
		fx.load_level_by_index(level_index);
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
			if self.menu.menu.is_none() {
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
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();

		fx.init();
		fx.load_level_by_index(level_index);
	}

	pub fn sync(&mut self) {
		let events = mem::replace(&mut self.menu.events, Vec::new());
		for evt in events {
			dbg!(&evt);
			match evt {
				menu::MenuEvent::NewGame => {
					self.play_level(1);
				}
				menu::MenuEvent::BackToMainMenu => {
					self.fx = None;
					self.menu.menu = Some(menu::Menu::Main(menu::MainMenu::default()));
				}
				menu::MenuEvent::NextLevel => {
					let level_index = if let Some(fx) = &self.fx { fx.level_index + 1 } else { 1 };
					self.play_level(level_index);
				}
				menu::MenuEvent::Retry => {
					let level_index = if let Some(fx) = &self.fx { fx.level_index } else { 1 };
					self.play_level(level_index);
				}
				menu::MenuEvent::Resume => {
					self.menu.close_all();
					if let Some(fx) = &mut self.fx {
						fx.unpause();
					}
				}
				_ => unimplemented!("{:?}", evt),
			}
		}

		if let Some(fx) = &mut self.fx {
			let events = mem::replace(&mut fx.events, Vec::new());
			for evt in events {
				dbg!(&evt);
				match evt {
					fx::FxEvent::PlaySound { sound } => {
						self.events.push(PlayEvent::PlaySound { sound });
					}
					fx::FxEvent::PlayMusic { music } => {
						self.events.push(PlayEvent::PlayMusic { music });
					}
					fx::FxEvent::Pause => {
						self.menu.menu = Some(menu::Menu::Pause(menu::PauseMenu::default()));
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
						self.menu.menu = Some(menu::Menu::Finished(menu));
					}
					_ => {}
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
			fx.draw(g, resx);
		}
		self.menu.draw(g, resx);
	}
}
