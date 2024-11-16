use std::{fmt, mem};
use cvmath::*;
use shade::d2::layout;
use crate::fx::Resources;

mod draw;
mod event;
mod main;
mod gamewin;
mod gameover;
mod pause;
mod options;
mod levelselect;
mod unlocklevel;
mod about;
mod u;
mod v;

pub use self::event::*;
pub use self::main::*;
pub use self::gamewin::*;
pub use self::gameover::*;
pub use self::pause::*;
pub use self::options::*;
pub use self::levelselect::*;
pub use self::unlocklevel::*;
pub use self::about::*;
pub use self::u::*;
pub use self::v::*;

const FONT_SIZE: f32 = 1.0 / 20.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyState { Release, Press, Down, Up }

impl KeyState {
	pub fn w(prev: bool, cur: bool) -> KeyState {
		match (prev, cur) {
			(false, false) => KeyState::Up,
			(false, true) => KeyState::Press,
			(true, false) => KeyState::Release,
			(true, true) => KeyState::Down,
		}
	}
	pub fn is_pressed(&self) -> bool {
		matches!(self, KeyState::Press)
	}
	pub fn is_held(&self) -> bool {
		matches!(self, KeyState::Press | KeyState::Down)
	}
}

pub struct Input {
	pub up: KeyState,
	pub left: KeyState,
	pub down: KeyState,
	pub right: KeyState,
	pub a: KeyState,
	pub b: KeyState,
	pub start: KeyState,
	pub select: KeyState,
}

pub enum Menu {
	Main(MainMenu),
	GameWin(GameWinMenu),
	GameOver(GameOverMenu),
	Pause(PauseMenu),
	Options(OptionsMenu),
	LevelSelect(levelselect::LevelSelectMenu),
	UnlockLevel(unlocklevel::UnlockLevelMenu),
	About(AboutMenu),
}
impl Menu {
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		match self {
			Menu::Main(menu) => menu.think(input, events),
			Menu::GameWin(menu) => menu.think(input, events),
			Menu::GameOver(menu) => menu.think(input, events),
			Menu::Pause(menu) => menu.think(input, events),
			Menu::Options(menu) => menu.think(input, events),
			Menu::LevelSelect(menu) => menu.think(input, events),
			Menu::UnlockLevel(menu) => menu.think(input, events),
			Menu::About(menu) => menu.think(input, events),
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		match self {
			Menu::Main(menu) => menu.draw(g, resx),
			Menu::GameWin(menu) => menu.draw(g, resx),
			Menu::GameOver(menu) => menu.draw(g, resx),
			Menu::Pause(menu) => menu.draw(g, resx),
			Menu::Options(menu) => menu.draw(g, resx),
			Menu::LevelSelect(menu) => menu.draw(g, resx),
			Menu::UnlockLevel(menu) => menu.draw(g, resx),
			Menu::About(menu) => menu.draw(g, resx),
		}
	}
}

#[derive(Default)]
pub struct MenuState {
	pub stack: Vec<Menu>,
	pub events: Vec<MenuEvent>,
}

impl MenuState {
	pub fn think(&mut self, input: &Input) {
		if let Some(menu) = self.stack.last_mut() {
			menu.think(input, &mut self.events);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		if let Some(menu) = self.stack.last_mut() {
			menu.draw(g, resx);
		}
	}
	pub fn close_all(&mut self) {
		self.stack.clear();
	}
	pub fn close_menu(&mut self) {
		let _ = self.stack.pop();
	}
	pub fn open_main(&mut self, start_from_continue: bool, title: &str) {
		self.stack.clear();
		let menu = MainMenu {
			title: title.to_string(),
			selected: if start_from_continue { 1 } else { 0 },
		};
		self.stack.push(Menu::Main(menu));
	}
}

fn foo(from: Rect<f32>, to: Rect<f32>) -> Transform2<f32> {
	let sx = (to.maxs.x - to.mins.x) / (from.maxs.x - from.mins.x);
	let sy = (to.maxs.y - to.mins.y) / (from.maxs.y - from.mins.y);
	Transform2 {
		a11: sx, a12: 0.0, a13: to.mins.x - from.mins.x * sx,
		a21: 0.0, a22: sy, a23: to.mins.y - from.mins.y * sy,
	}
}

pub fn darken(g: &mut shade::Graphics, resx: &Resources, alpha: u8) {
	let mut cv = shade::d2::CommandBuffer::<UiVertex, UiUniform>::new();

	cv.blend_mode = shade::BlendMode::Alpha;
	cv.shader = resx.colorshader;
	cv.viewport = cvmath::Rect::vec(resx.screen_size);

	let paint = shade::d2::Paint {
		template: UiVertex { pos: Vec2::ZERO, uv: Vec2::ZERO, color: [0, 0, 0, alpha] },
	};
	cv.fill_rect(&paint, &cvmath::Rect::c(-1.0, 1.0, 1.0, -1.0));

	cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
}

fn wrap_items<'a, const N: usize>(items: &'a [&'a str; N]) -> [&'a (dyn fmt::Display + 'a); N] {
	items.each_ref().map(|item| item as &dyn fmt::Display)
}
