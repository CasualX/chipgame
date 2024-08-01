use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	Game(core::GameEvent),
}

pub enum MenuState {
	Main(fx::MainMenu),
	Finished(fx::GameWinMenu),
}

impl MenuState {
	pub fn think(&mut self, input: &core::Input) {
		match self {
			MenuState::Main(menu) => menu.think(input),
			MenuState::Finished(menu) => menu.think(input),
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources) {
		match self {
			MenuState::Main(menu) => menu.draw(g, resx),
			MenuState::Finished(menu) => menu.draw(g, resx),
		}
	}
}

#[derive(Default)]
pub struct PlayState {
	pub fx: Option<fx::FxState>,
	pub menu: Option<MenuState>,
}

impl PlayState {
	pub fn think(&mut self, input: &core::Input) {
		if let Some(menu) = &mut self.menu {
			menu.think(input);
		}
		else if let Some(fx) = &mut self.fx {
			fx.think(input, None);
		}
	}

	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources) {
		if let Some(fx) = &mut self.fx {
			fx.draw(g, resx);
		}
		if let Some(menu) = &mut self.menu {
			menu.draw(g, resx);
		}
	}
}
