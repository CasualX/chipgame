use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct InputDir {
	dir: Compass,
	seen: bool,
}

/// Input buffering and socd handling.
#[derive(Clone, Debug)]
pub struct InputBuffer {
	moves: [InputDir; 4],
	nmoves: u8,
}

impl Default for InputBuffer {
	fn default() -> Self {
		Self {
			moves: [InputDir { dir: Compass::Up, seen: false }; 4],
			nmoves: 0,
		}
	}
}

impl InputBuffer {
	#[inline(never)]
	pub fn handle(&mut self, dir: Compass, is: bool, was: bool) {
		if !was && is {
			self.add_dir(dir);
		}
		if /*was && */!is {
			self.remove_dir(dir);
		}
	}

	fn add_dir(&mut self, dir: Compass) {
		// Find the first seen move
		let mut i = 0;
		let nmoves = cmp::min(self.nmoves, 4);
		if i < nmoves && !self.moves[i as usize].seen {
			i += 1;
			if i < nmoves && !self.moves[i as usize].seen {
				i += 1;
				if i < nmoves && !self.moves[i as usize].seen {
					i += 1;
				}
			}
		}

		self.nmoves = cmp::min(nmoves + 1, 4);

		// Shift seen moves to the right
		if i <= 2 {
			self.moves[3] = self.moves[2];
			if i <= 1 {
				self.moves[2] = self.moves[1];
				if i <= 0 {
					self.moves[1] = self.moves[0];
				}
			}
		}

		// Write the new move
		self.moves[i as usize] = InputDir { dir, seen: false };
	}

	fn remove_dir(&mut self, dir: Compass) {
		if self.nmoves > 3 && self.moves[3] == (InputDir { dir, seen: true }) {
			self.nmoves -= 1;
		}
		if self.nmoves > 2 && self.moves[2] == (InputDir { dir, seen: true }) {
			self.moves[2] = self.moves[3];
			self.nmoves -= 1;
		}
		if self.nmoves > 1 && self.moves[1] == (InputDir { dir, seen: true }) {
			self.moves[1] = self.moves[2];
			self.moves[2] = self.moves[3];
			self.nmoves -= 1;
		}
		if self.nmoves > 0 && self.moves[0] == (InputDir { dir, seen: true }) {
			self.moves[0] = self.moves[1];
			self.moves[1] = self.moves[2];
			self.moves[2] = self.moves[3];
			self.nmoves -= 1;
		}
	}

	pub fn read_dir(&mut self) -> Option<Compass> {
		if self.nmoves > 0 {
			self.moves[0].seen = true;
			Some(self.moves[0].dir)
		}
		else {
			None
		}
	}
}
