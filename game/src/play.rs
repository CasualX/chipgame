use std::mem;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<MusicId> },
}

pub enum MenuState {
	Main(menu::MainMenu),
	Finished(menu::GameWinMenu),
	Pause(menu::PauseMenu),
}

impl MenuState {
	pub fn think(&mut self, input: &core::Input) {
		match self {
			MenuState::Main(menu) => menu.think(input),
			MenuState::Finished(menu) => menu.think(input),
			MenuState::Pause(menu) => menu.think(input),
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources) {
		match self {
			MenuState::Main(menu) => menu.draw(g, resx),
			MenuState::Finished(menu) => menu.draw(g, resx),
			MenuState::Pause(menu) => menu.draw(g, resx),
		}
	}
}

#[derive(Default)]
pub struct PlayState {
	pub fx: Option<fx::FxState>,
	pub menu: Option<MenuState>,
	pub events: Vec<PlayEvent>,
}

impl PlayState {
	pub fn launch(&mut self) {
		self.menu = Some(MenuState::Main(menu::MainMenu::default()));
	}

	pub fn load_level(&mut self, level_index: i32) {
		self.menu = None;
		self.fx = Some(fx::FxState::default());
		let fx = self.fx.as_mut().unwrap();

		fx.init();
		fx.load_level_by_index(level_index);
	}

	pub fn think(&mut self, input: &core::Input) {
		if let Some(menu) = &mut self.menu {
			menu.think(input);
		}
		else if let Some(fx) = &mut self.fx {
			fx.think(input);
		}
		self.sync();
	}

	pub fn sync(&mut self) {
		let events = match self.menu {
			Some(MenuState::Main(ref mut menu)) => mem::replace(&mut menu.events, Vec::new()),
			Some(MenuState::Finished(ref mut menu)) => mem::replace(&mut menu.events, Vec::new()),
			Some(MenuState::Pause(ref mut menu)) => mem::replace(&mut menu.events, Vec::new()),
			None => Vec::new(),
		};
		for evt in events {
			match evt {
				menu::MenuEvent::NewGame => {
					self.menu = None;
					self.fx = Some(fx::FxState::default());
					let fx = self.fx.as_mut().unwrap();

					fx.init();
					fx.load_level_by_index(1);
				}
				_ => unimplemented!("{:?}", evt),
			}
		}

		if let Some(fx) = &mut self.fx {
			let events = mem::replace(&mut fx.events, Vec::new());
			for evt in events {
				match evt {
					fx::FxEvent::PlaySound { sound } => {
						self.events.push(PlayEvent::PlaySound { sound });
					}
					fx::FxEvent::PlayMusic { music } => {
						self.events.push(PlayEvent::PlayMusic { music });
					}
					fx::FxEvent::Pause => {
						self.menu = Some(MenuState::Pause(menu::PauseMenu::default()));
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
		if let Some(menu) = &mut self.menu {
			menu.draw(g, resx);
		}
	}
}
