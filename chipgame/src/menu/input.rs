
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
