
pub struct XInput {
	handle: rusty_xinput::XInputHandle,
}

impl XInput {
	pub fn new() -> XInput {
		let handle = rusty_xinput::XInputHandle::load_default().unwrap();
		XInput { handle }
	}

	pub fn get_state(&self, input: &mut chipcore::Input) {
		for user_index in 0..4 {
			self.get_state_(user_index, input);
		}
	}

	fn get_state_(&self, user_index: u32, input: &mut chipcore::Input) {
		if let Ok(state) = self.handle.get_state(user_index) {
			input.up |= state.arrow_up();
			input.down |= state.arrow_down();
			input.left |= state.arrow_left();
			input.right |= state.arrow_right();
			input.a |= state.south_button();
			input.b |= state.east_button();
			input.start |= state.start_button();
			input.select |= state.select_button();
		}
	}
}
