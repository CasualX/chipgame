
const LIMIT: usize = 100;

/// Simple history stack for undo/redo functionality.
#[derive(Clone, Debug, Default)]
pub struct History<T> {
	entries: Vec<T>,
	cursor: usize,
}

impl<T> History<T> {
	/// Create a new history with given initial state.
	pub fn new(init: T) -> Self {
		History { entries: vec![init], cursor: 0 }
	}

	/// Clear the history.
	pub fn clear(&mut self, init: T) {
		self.entries.clear();
		self.entries.push(init);
		self.cursor = 0;
	}

	/// Push a new entry onto the history only if it differs from the current entry.
	pub fn push_if(&mut self, value: T) where T: PartialEq {
		if let Some(current) = self.entries.get(self.cursor) {
			if *current == value {
				return;
			}
		}

		self.push(value);
	}

	/// Push a new entry onto the history, truncating any "redo" entries.
	pub fn push(&mut self, value: T) {
		if self.cursor + 1 < self.entries.len() {
			self.entries.truncate(self.cursor + 1);
		}

		while self.entries.len() >= LIMIT {
			self.entries.remove(0);
			if self.cursor > 0 {
				self.cursor -= 1;
			}
		}

		self.entries.push(value);
		self.cursor = self.entries.len() - 1;
	}

	/// Undo the last action, returning the previous entry if available.
	pub fn undo(&mut self) -> Option<&T> {
		if self.cursor == 0 {
			return None;
		}
		self.cursor -= 1;
		self.entries.get(self.cursor)
	}

	/// Redo the last undone action, returning the next entry if available.
	pub fn redo(&mut self) -> Option<&T> {
		let entry = self.entries.get(self.cursor + 1)?;
		self.cursor += 1;
		Some(entry)
	}
}
