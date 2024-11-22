use super::*;

#[derive(Default)]
pub struct UnlockLevelMenu {
	pub selected: i8,
	pub password: [Option<char>; 4],
}

const LETTERS: &[&[u8]] = &[
	[b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P'].as_slice(),
	   [b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L'].as_slice(),
	         [b'Z', b'X', b'C', b'V', b'B', b'N', b'M'].as_slice(),
];

fn get_row_index(selected: i8) -> usize {
	if selected < LETTERS[0].len() as i8 { 0 }
	else if selected < (LETTERS[0].len() + LETTERS[1].len()) as i8 { 1 }
	else { 2 }
}

impl UnlockLevelMenu {
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			match get_row_index(self.selected) {
				0 => (),
				1 => self.selected -= 10,
				2 => self.selected -= 8,
				_ => unreachable!(),
			}
		}

		if input.down.is_pressed() {
			match get_row_index(self.selected) {
				0 => self.selected += 10,
				1 => self.selected += 8,
				2 => (),
				_ => unreachable!(),
			}
		}

		if input.left.is_pressed() {
			if self.selected > 0 {
				self.selected -= 1;
			}
			else {
				self.selected = 25;
			}
		}

		if input.right.is_pressed() {
			if self.selected < 25 {
				self.selected += 1;
			}
			else {
				self.selected = 0;
			}
		}

		if input.a.is_pressed() {
			if let Some(slot) = self.password.iter().position(|&x| x.is_none()) {
				self.password[slot] = match get_row_index(self.selected) {
					0 => Some(LETTERS[0][self.selected as usize] as char),
					1 => Some(LETTERS[1][self.selected as usize - LETTERS[0].len()] as char),
					2 => Some(LETTERS[2][self.selected as usize - LETTERS[0].len() - LETTERS[1].len()] as char),
					_ => unreachable!(),
				}
			}
		}

		if input.b.is_pressed() {
			if let Some(slot) = self.password.iter().rposition(|&x| x.is_some()) {
				self.password[slot] = None;
			}
			else {
				events.push(MenuEvent::CloseMenu);
			}
		}

		if input.start.is_pressed() {
			if let [Some(_1), Some(_2), Some(_3), Some(_4)] = self.password {
				events.push(MenuEvent::EnterPassword { code: [_1 as u8, _2 as u8, _3 as u8, _4 as u8] });
			}
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

		buf.text_lines(&resx.font, &scribe, &rect, shade::d2::BoxAlign::TopCenter, &[
			&format_args!("Enter Password: {} {} {} {}", self.password[0].unwrap_or('_'), self.password[1].unwrap_or('_'), self.password[2].unwrap_or('_'), self.password[3].unwrap_or('_')),
		]);

		let height = LETTERS.len() as f32 * size * 1.5;
		for (i, &line) in LETTERS.iter().enumerate() {
			let y = (resx.screen_size.y as f32 - height) * 0.5 + i as f32 * size * 1.5;
			let width = line.len() as f32 * size * 1.5;
			for (j, &chr) in line.iter().enumerate() {
				let xstart = (resx.screen_size.x as f32 - width) * 0.5;
				let current_index = match i { 0 => j as i8, 1 => j as i8 + 10, 2 => j as i8 + 19, _ => unreachable!() };
				let rect = cvmath::Rect::c(xstart + j as f32 * size * 1.5, y, xstart + j as f32 * size * 1.5, y);
				let scribe = shade::d2::Scribe {
					font_size: size,
					line_height: size * (5.0 / 4.0),
					color: if current_index == self.selected { cvmath::Vec4(255, 255, 255, 255) } else { cvmath::Vec4(128, 128, 128, 255) },
					..Default::default()
				};
				let chr = chr as char;
				buf.text_lines(&resx.font, &scribe, &rect, shade::d2::BoxAlign::MiddleCenter, &[&chr]);
			}
		}

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
