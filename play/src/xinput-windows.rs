

pub struct XInput {
	handle: rusty_xinput::XInputHandle,
}

impl XInput {
	pub fn new() -> XInput {
		let handle = rusty_xinput::XInputHandle::load_default().unwrap();
	}

	pub fn get_state(&self, user_index: u32, input: &mut chipgame::core::Input) {
		let state = self.handle.get_state(user_index).unwrap();
		input.up = state.north_button() || state.arrow_up();
		input.down = state.south_button() || state.arrow_down();
		input.left = state.west_button() || state.arrow_left();
		input.right = state.east_button() || state.arrow_right();
		input.start = state.start_button();
		input.select = state.select_button();
	}
}
