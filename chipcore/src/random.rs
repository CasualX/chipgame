use super::*;

/// Source of game randomness.
#[repr(transparent)]
pub struct Random {
	rand: urandom::Random<urandom::rng::Xoshiro256>,
}

impl ops::Deref for Random {
	type Target = urandom::Random<urandom::rng::Xoshiro256>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.rand
	}
}

impl ops::DerefMut for Random {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.rand
	}
}

impl Default for Random {
	fn default() -> Self {
		Random {
			rand: urandom::rng::Xoshiro256::new(),
		}
	}
}

impl Random {
	pub fn reseed(&mut self, seed: u64) {
		self.rand = urandom::rng::Xoshiro256::from_seed(seed);
	}

	pub fn compass(&mut self) -> Compass {
		match self.rand.next_u32() % 4 {
			0 => Compass::Up,
			1 => Compass::Left,
			2 => Compass::Down,
			_ => Compass::Right,
		}
	}
}
