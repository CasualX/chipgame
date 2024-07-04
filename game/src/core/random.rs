use super::*;

/// Source of game randomness.
pub struct Random {
	/// Random number generator.
	pub rng: urandom::Random<urandom::rng::Xoshiro256>,
}
impl Default for Random {
	fn default() -> Self {
		Random {
			rng: urandom::rng::Xoshiro256::new(),
		}
	}
}

impl Random {
	pub fn compass(&mut self) -> Compass {
		match self.rng.next_u32() % 4 {
			0 => Compass::Up,
			1 => Compass::Left,
			2 => Compass::Down,
			_ => Compass::Right,
		}
	}
}
