
pub struct GamepadManager {
	gilrs: Option<gilrs::Gilrs>,
}

impl GamepadManager {
	pub fn new() -> GamepadManager {
		let gilrs = match gilrs::Gilrs::new() {
			Ok(g) => Some(g),
			Err(err) => {
				eprintln!("[gamepad] failed to initialize gilrs: {err}");
				None
			}
		};
		GamepadManager { gilrs }
	}

	pub fn poll(&mut self) -> chipcore::Input {
		let mut aggregated = chipcore::Input::default();
		let Some(gilrs) = self.gilrs.as_mut() else {
			return aggregated;
		};

		while let Some(event) = gilrs.next_event() {
			use gilrs::EventType;
			match event.event {
				EventType::Disconnected => {
					// Drain events so gilrs updates its internal state.
				}
				EventType::Connected => {
					// Same as above; nothing to do for now.
				}
				_ => {}
			}
		}

		for (_id, gamepad) in gilrs.gamepads() {
			if !gamepad.is_connected() {
				continue;
			}

			aggregated.up |= gamepad.is_pressed(gilrs::Button::DPadUp);
			aggregated.down |= gamepad.is_pressed(gilrs::Button::DPadDown);
			aggregated.left |= gamepad.is_pressed(gilrs::Button::DPadLeft);
			aggregated.right |= gamepad.is_pressed(gilrs::Button::DPadRight);

			aggregated.a |= gamepad.is_pressed(gilrs::Button::South);
			aggregated.b |= gamepad.is_pressed(gilrs::Button::East);
			aggregated.start |= gamepad.is_pressed(gilrs::Button::Start);
			aggregated.select |= gamepad.is_pressed(gilrs::Button::Select) || gamepad.is_pressed(gilrs::Button::Mode);

			apply_axis(&gamepad, gilrs::Axis::LeftStickX, &mut aggregated.left, &mut aggregated.right, false);
			apply_axis(&gamepad, gilrs::Axis::LeftStickY, &mut aggregated.up, &mut aggregated.down, true);
			apply_axis(&gamepad, gilrs::Axis::DPadX, &mut aggregated.left, &mut aggregated.right, false);
			apply_axis(&gamepad, gilrs::Axis::DPadY, &mut aggregated.up, &mut aggregated.down, false);
		}

		aggregated
	}
}

fn apply_axis(gamepad: &gilrs::Gamepad, axis: gilrs::Axis, negative: &mut bool, positive: &mut bool, invert: bool) {
	const DEADZONE: f32 = 0.5;

	if let Some(axis_data) = gamepad.axis_data(axis) {
		let mut value = axis_data.value();
		if invert {
			value = -value;
		}
		if value <= -DEADZONE {
			*negative = true;
		}
		if value >= DEADZONE {
			*positive = true;
		}
	}
}
