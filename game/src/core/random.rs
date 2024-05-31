
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
