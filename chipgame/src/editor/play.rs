use super::*;

pub struct EditorPlayState {
	pub level: String,
	pub game: fx::FxState,
	pub input: Input,
	pub screen_size: Vec2i,
}

impl EditorPlayState {
	pub fn init(&mut self) {
		self.game.render.tiles = &crate::play::tiles::TILES_PLAY;
	}
	pub fn key_left(&mut self, pressed: bool) {
		self.input.key_left = pressed;
	}
	pub fn key_right(&mut self, pressed: bool) {
		self.input.key_right = pressed;
	}
	pub fn key_up(&mut self, pressed: bool) {
		self.input.key_up = pressed;
	}
	pub fn key_down(&mut self, pressed: bool) {
		self.input.key_down = pressed;
	}
	pub fn set_screen_size(&mut self, width: i32, height: i32) {
		self.screen_size = Vec2::new(width, height);
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources) {
		// Clear the screen
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});

		let input = menu::Input {
			a: menu::KeyState::Up,
			b: menu::KeyState::Up,
			up: if self.input.key_up { menu::KeyState::Down } else { menu::KeyState::Up },
			down: if self.input.key_down { menu::KeyState::Down } else { menu::KeyState::Up },
			left: if self.input.key_left { menu::KeyState::Down } else { menu::KeyState::Up },
			right: if self.input.key_right { menu::KeyState::Down } else { menu::KeyState::Up },
			start: menu::KeyState::Up,
			select: menu::KeyState::Up,
		};
		self.game.think(&input);
		self.game.draw(g, resx);
		return;
	}
}
