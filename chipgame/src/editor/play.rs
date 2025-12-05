use super::*;

pub struct EditorPlayState {
	pub level: String,
	pub game: Box<fx::FxState>,
	pub input: Input,
	pub screen_size: Vec2i,
}

impl EditorPlayState {
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
	pub fn think(&mut self) {
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
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &fx::Resources, time: f64) {
		g.clear(&shade::ClearArgs {
			surface: shade::Surface::BACK_BUFFER,
			color: Some(Vec4(0.2, 0.2, 0.5, 1.0)),
			depth: Some(1.0),
			..Default::default()
		});
		self.game.draw(g, resx, time);
	}
	pub fn save_replay(&mut self) {
		let replay_dto: chipty::ReplayDto = self.game.gs.save_replay(self.game.game_realtime);
		let mut level_dto: chipty::LevelDto = serde_json::from_str(&self.level).unwrap();
		level_dto.replays = Some(vec![replay_dto]);
		self.level = serde_json::to_string(&level_dto).unwrap();
	}
	pub fn play_stats(&self) -> EditorPlayStats {
		EditorPlayStats {
			realtime: self.game.game_realtime,
			ticks: self.game.gs.time,
			steps: self.game.gs.ps.steps,
			bonks: self.game.gs.ps.bonks,
		}
	}
}
