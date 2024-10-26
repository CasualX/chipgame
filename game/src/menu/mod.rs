use std::mem;
use cvmath::*;
use crate::fx::Resources;

mod event;
mod main;
mod gamewin;
mod pausemenu;
mod u;
mod v;

pub use self::event::*;
pub use self::main::*;
pub use self::gamewin::*;
pub use self::pausemenu::*;
pub use self::u::*;
pub use self::v::*;

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
	Finished(GameWinMenu),
	Pause(PauseMenu),
}
impl Menu {
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		match self {
			Menu::Main(menu) => menu.think(input, events),
			Menu::Finished(menu) => menu.think(input, events),
			Menu::Pause(menu) => menu.think(input, events),
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		match self {
			Menu::Main(menu) => menu.draw(g, resx),
			Menu::Finished(menu) => menu.draw(g, resx),
			Menu::Pause(menu) => menu.draw(g, resx),
		}
	}
}

#[derive(Default)]
pub struct MenuState {
	pub menu: Option<Menu>,
	pub events: Vec<MenuEvent>,
}

impl MenuState {
	pub fn think(&mut self, input: &Input) {
		if let Some(menu) = &mut self.menu {
			menu.think(input, &mut self.events);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		if let Some(menu) = &mut self.menu {
			menu.draw(g, resx);
		}
	}
	pub fn close_all(&mut self) {
		self.menu = None;
	}
	pub fn open_main(&mut self) {
		self.menu = Some(Menu::Main(MainMenu::default()));
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

fn darken(g: &mut shade::Graphics, resx: &Resources, alpha: u8) {
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
