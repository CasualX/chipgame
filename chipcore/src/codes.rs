use super::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct CodeSequenceState {
	pos: u8,
}

impl CodeSequenceState {
	pub fn next(&mut self, btn: Button, code: &[Button]) -> bool {
		if code.get(self.pos as usize) == Some(&btn) {
			self.pos += 1;
			if self.pos as usize == code.len() {
				self.pos = 0;
				return true;
			}
		}
		else {
			self.pos = 0;
		}
		return false;
	}
}

pub(super) static CODE_GIVEALL: [Button; 10] = { use Button::*; [Up, Up, Down, Down, Left, Right, Left, Right, B, A] };
pub(super) static CODE_WTW: [Button; 6] = { use Button::*; [A, B, Up, A, B, Down] };
pub(super) static CODE_INFTIME: [Button; 7] = { use Button::*; [A, Up, Right, Down, Left, Up, A] };
pub(super) static CODE_WIN: [Button; 6] = { use Button::*; [A, A, A, B, B, B] };
