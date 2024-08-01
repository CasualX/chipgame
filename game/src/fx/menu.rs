use super::*;

fn foo(from: Rect<f32>, to: Rect<f32>) -> Transform2<f32> {
	let sx = (to.maxs.x - to.mins.x) / (from.maxs.x - from.mins.x);
	let sy = (to.maxs.y - to.mins.y) / (from.maxs.y - from.mins.y);
	Transform2 {
		a11: sx, a12: 0.0, a13: to.mins.x - from.mins.x * sx,
		a21: 0.0, a22: sy, a23: to.mins.y - from.mins.y * sy,
	}
}

fn darken(g: &mut shade::Graphics, resx: &Resources, alpha: u8) {
	let mut cv = shade::d2::CommandBuffer::<ui::UiVertex, ui::UiUniform>::new();

	cv.blend_mode = shade::BlendMode::Alpha;
	cv.shader = resx.colorshader;
	cv.viewport = cvmath::Rect::vec(resx.screen_size);

	let paint = shade::d2::Paint {
		template: ui::UiVertex { pos: Vec2::ZERO, uv: Vec2::ZERO, color: [0, 0, 0, alpha] },
	};
	cv.fill_rect(&paint, &cvmath::Rect::c(-1.0, 1.0, 1.0, -1.0));

	cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuEvent {
	// Main Events
	CursorMove,
	NewGame,
	GoToLevel,
	HighScores,
	Options,
	About,
	Exit,
	BackToMainMenu,

	// Pause Events
	Resume,
	Restart,
	BackToPauseMenu,

	// Options Events
	BackgroundMusicOn,
	BackgroundMusicOff,
	SoundEffectsOn,
	SoundEffectsOff,
	DevModeOn,
	DevModeOff,

	// Level select Events
	UnlockLevel,
	SelectLevel { level_index: i32 },

	// Unlock level Events
	EnterPassword { code: [u8; 4] },
	BackToLevelSelect,

	// Game Over Events
	NextLevel,
	Retry,
}

#[derive(Default)]
pub struct MainMenu {
	pub selected: u8,
	pub input: core::Input,
}

impl MainMenu {
	const ITEMS: [&'static str; 7] = ["New game", "Continue", "Go to level", "High scores", "Options", "About", "Exit"];
	pub fn think(&mut self, input: &core::Input) {
		if input.up && !self.input.up {
			self.selected = if self.selected > 0 { self.selected - 1 } else { self.selected };
		}
		if input.down && !self.input.down {
			self.selected = if self.selected < Self::ITEMS.len() as u8 - 1 { self.selected + 1 } else { self.selected };
		}
		self.input = *input;
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		let mut buf = shade::d2::TextBuffer::new();
		buf.shader = resx.font.shader;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.viewport = cvmath::Rect::vec(resx.screen_size);

		let ss = resx.screen_size;
		let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));

		buf.push_uniform(shade::d2::TextUniform {
			transform,
			texture: resx.font.texture,
			..Default::default()
		});

		// let mut pos = Vec2::ZERO;
		let mut scribe = shade::d2::Scribe {
			font_size: 32.0,
			line_height: 40.0,
			..Default::default()
		};

		for (i, item) in Self::ITEMS.iter().enumerate() {
			let color = if i == self.selected as usize { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 * 0.5 - 100.0 + i as i32 as f32 * scribe.line_height));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, item);
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}

#[derive(Default)]
pub struct GameWinMenu {
	pub selected: u8,
	pub level_index: i32,
	pub level_name: String,
	pub attempts: i32,
	pub time: i32,
	pub steps: i32,
	pub bonks: i32,
	pub input: core::Input,
	pub events: Vec<MenuEvent>,
}

impl GameWinMenu {
	const ITEMS: [&'static str; 3] = ["Next level", "Retry", "Main menu"];
	pub fn think(&mut self, input: &core::Input) {
		if input.up && !self.input.up {
			if self.selected > 0 {
				self.selected -= 1;
				self.events.push(MenuEvent::CursorMove);
			}
		}
		if input.down && !self.input.down {
			if self.selected < Self::ITEMS.len() as u8 - 1 {
				self.selected += 1;
				self.events.push(MenuEvent::CursorMove);
			}
		}
		if input.a && !self.input.a || input.start && !self.input.start {
			let evt = match self.selected {
				0 => MenuEvent::NextLevel,
				1 => MenuEvent::Retry,
				_ => MenuEvent::BackToMainMenu,
			};
			self.events.push(evt);
		}
		self.input = *input;
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		darken(g, resx, 128);

		let mut buf = shade::d2::TextBuffer::new();
		buf.shader = resx.font.shader;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.viewport = cvmath::Rect::vec(resx.screen_size);

		let ss = resx.screen_size;
		let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));

		buf.push_uniform(shade::d2::TextUniform {
			transform,
			texture: resx.font.texture,
			..Default::default()
		});

		let size = resx.screen_size.y as f32 / 20.0;

		let mut scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, size * 3.0));
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, &format!("Level {}: {}", self.level_index, self.level_name));
		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, size * 3.0 + scribe.line_height));
		scribe.color = cvmath::Vec4(0, 255, 128, 255);
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, "Finished!");

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5 - size * 4.0, resx.screen_size.y as f32 * 0.5));
		scribe.color = cvmath::Vec4(255, 255, 255, 255);
		buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleLeft, "Attempts:\nTime:\nSteps:\nBonks:");

		let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5 + size * 4.0, resx.screen_size.y as f32 * 0.5));
		scribe.color = cvmath::Vec4(0, 255, 128, 255);
		let frames = self.time % 60;
		let seconds = (self.time / 60) % 60;
		let minutes = self.time / 3600;
		if minutes > 0 {
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleRight, &format!("{}\n{}:{:02}.{:02}\n{}\n{}", self.attempts, minutes, seconds, frames, self.steps, self.bonks));
		}
		else {
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleRight, &format!("{}\n{}.{:02}\n{}\n{}", self.attempts, seconds, frames, self.steps, self.bonks));
		}

		// let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 - size * 3.0));
		// scribe.color = cvmath::Vec4(255, 255, 255, 255);
		// buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::BottomCenter, "Next level\nRetry\nMain menu");
		for (i, item) in Self::ITEMS.iter().enumerate() {
			let color = if i == self.selected as usize { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) };
			scribe.color = color;

			let rect = cvmath::Rect::point(Vec2(resx.screen_size.x as f32 * 0.5, resx.screen_size.y as f32 - size * (2.0 + Self::ITEMS.len() as f32) + i as i32 as f32 * scribe.line_height));
			buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, item);
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
