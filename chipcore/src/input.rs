use std::ops;

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
	pub const NEUTRAL: Input = Input {
		up: false,
		left: false,
		down: false,
		right: false,
		a: false,
		b: false,
		start: false,
		select: false,
	};

	pub const LEFT: Input = Input { left: true, ..Input::NEUTRAL };
	pub const RIGHT: Input = Input { right: true, ..Input::NEUTRAL };
	pub const UP: Input = Input { up: true, ..Input::NEUTRAL };
	pub const DOWN: Input = Input { down: true, ..Input::NEUTRAL };

	/// Returns true if any directional input is pressed.
	pub const fn has_directional_input(&self) -> bool {
		self.up | self.left | self.down | self.right
	}

	/// Encodes the input state into a single byte.
	#[inline]
	pub const fn encode(&self) -> u8 {
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
		bits
	}

	/// Decodes an input state from a single byte.
	#[inline]
	pub const fn decode(byte: u8) -> Input {
		Input {
			up: byte & INPUT_UP != 0,
			left: byte & INPUT_LEFT != 0,
			down: byte & INPUT_DOWN != 0,
			right: byte & INPUT_RIGHT != 0,
			a: byte & INPUT_A != 0,
			b: byte & INPUT_B != 0,
			start: byte & INPUT_START != 0,
			select: byte & INPUT_SELECT != 0,
		}
	}
}

impl ops::BitOr<Input> for Input {
	type Output = Input;

	fn bitor(self, rhs: Input) -> Input {
		Input {
			up: self.up | rhs.up,
			left: self.left | rhs.left,
			down: self.down | rhs.down,
			right: self.right | rhs.right,
			a: self.a | rhs.a,
			b: self.b | rhs.b,
			start: self.start | rhs.start,
			select: self.select | rhs.select,
		}
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
