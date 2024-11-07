
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Button { Up, Down, Left, Right, A, B, Start, Select }

/// Input data.
#[derive(Copy, Clone, Debug, Default)]
pub struct Input {
	pub up: bool,
	pub left: bool,
	pub down: bool,
	pub right: bool,
	pub a: bool,
	pub b: bool,
	pub start: bool,
	pub select: bool,
}

impl Input {
	pub fn any_arrows(&self) -> bool {
		self.up | self.left | self.down | self.right
	}

	pub fn encode(&self, buf: &mut Vec<u8>) {
		let mut bits = 0;
		if self.up {
			bits |= INPUT_UP;
		}
		if self.left {
			bits |= INPUT_LEFT;
		}
		if self.down {
			bits |= INPUT_DOWN;
		}
		if self.right {
			bits |= INPUT_RIGHT;
		}
		if self.a {
			bits |= INPUT_A;
		}
		if self.b {
			bits |= INPUT_B;
		}
		if self.start {
			bits |= INPUT_START;
		}
		if self.select {
			bits |= INPUT_SELECT;
		}
		buf.push(bits);
	}
}

const INPUT_UP: u8 = 1 << 0;
const INPUT_LEFT: u8 = 1 << 1;
const INPUT_DOWN: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_A: u8 = 1 << 4;
const INPUT_B: u8 = 1 << 5;
const INPUT_START: u8 = 1 << 6;
const INPUT_SELECT: u8 = 1 << 7;
