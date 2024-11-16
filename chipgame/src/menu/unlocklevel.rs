use super::*;

#[derive(Default)]
pub struct UnlockLevelMenu {
	pub selected: i32,
	pub password: [Option<char>; 4],
}

const LETTERS: &[&[char]; 3] = &[
	['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'].as_slice(),
	['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'].as_slice(),
	['Z', 'X', 'C', 'V', 'B', 'N', 'M'].as_slice(),
];

impl UnlockLevelMenu {
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.b.is_pressed() {
			events.push(MenuEvent::CloseMenu);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {

		let mut buf = shade::d2::TextBuffer::new();
		buf.shader = resx.font.shader;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.viewport = cvmath::Rect::vec(resx.screen_size);

		let rect = Rect::vec(resx.screen_size.cast::<f32>());
		let transform = foo(rect, Rect::c(-1.0, 1.0, 1.0, -1.0));

		buf.push_uniform(shade::d2::TextUniform {
			transform,
			texture: resx.font.texture,
			..Default::default()
		});

		let size = resx.screen_size.y as f32 * FONT_SIZE;

		let scribe = shade::d2::Scribe {
			font_size: size,
			line_height: size * (5.0 / 4.0),
			color: cvmath::Vec4(255, 255, 255, 255),
			..Default::default()
		};

		let rect = cvmath::Rect::c(0.0, size + size, resx.screen_size.x as f32, size + size);

		buf.text_fmt_lines(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, &[
			format_args!("Enter Password: {} {} {} {}", self.password[0].unwrap_or('_'), self.password[1].unwrap_or('_'), self.password[2].unwrap_or('_'), self.password[3].unwrap_or('_')),
		]);

		let height = LETTERS.len() as f32 * size * 1.5;
		for (i, &line) in LETTERS.iter().enumerate() {
			let y = (resx.screen_size.y as f32 - height) * 0.5 + i as f32 * size * 1.5;
			let width = line.len() as f32 * size * 1.5;
			for (j, &chr) in line.iter().enumerate() {
				let xstart = (resx.screen_size.x as f32 - width) * 0.5;

				let rect = cvmath::Rect::c(xstart + j as f32 * size * 1.5, y, xstart + j as f32 * size * 1.5, y);
				let scribe = shade::d2::Scribe {
					font_size: size,
					line_height: size * (5.0 / 4.0),
					color: cvmath::Vec4(255, 255, 255, 255),
					..Default::default()
				};
				buf.text_box(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, &format!("{}", chr));
			}
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
