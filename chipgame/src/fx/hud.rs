use super::*;

use crate::menu::{UiUniform, UiVertex};

impl FxState {
	pub fn render_ui(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		let darken_time = f32::min(1.0, (self.time - self.darken_time) / 0.2);
		let alpha = if self.darken { darken_time } else { 1.0 - darken_time };
		if alpha > 0.0 {
			let alpha = (alpha * 0.625 * 255.0) as u8;
			crate::menu::darken(g, resx, alpha);
		}

		let mut pool = shade::d2::DrawPool::new();

		let cv = pool.get::<UiVertex, UiUniform>();
		cv.viewport = resx.viewport;
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.shader = resx.uishader;

		cv.uniform.transform = Transform2f::ortho(cv.viewport.cast());
		cv.uniform.texture = resx.texdigits;

		// let paint = shade::d2::Paint {
		// 	template: UiVertex {
		// 		pos: Vec2f::ZERO,
		// 		uv: Vec2f::ZERO,
		// 		color: [255, 255, 255, 255],
		// 	},
		// };
		let ss = resx.viewport.size();
		let a = ss.y as f32 * 0.075;
		// cv.fill_rect(&paint, &Bounds2::c(0.0, 0.0, ss.x as f32, a + a));

		cv.uniform.texture = resx.tileset;

		let ref gs = self.gs;

		if gs.ps.keys[0] > 0 {
			draw_sprite(cv, data::SpriteId::BlueKey, resx.tileset_size, Vec2(a * 0.0, 0.0), a);
		}
		if gs.ps.keys[1] > 0 {
			draw_sprite(cv, data::SpriteId::RedKey, resx.tileset_size, Vec2(a * 1.0, 0.0), a);
		}
		if gs.ps.keys[2] > 0 {
			draw_sprite(cv, data::SpriteId::GreenKey, resx.tileset_size, Vec2(a * 2.0, 0.0), a);
		}
		if gs.ps.keys[3] > 0 {
			draw_sprite(cv, data::SpriteId::YellowKey, resx.tileset_size, Vec2(a * 3.0, 0.0), a);
		}

		if gs.ps.flippers {
			draw_sprite(cv, data::SpriteId::Flippers, resx.tileset_size, Vec2(a * 0.0, a), a);
		}
		if gs.ps.fire_boots {
			draw_sprite(cv, data::SpriteId::FireBoots, resx.tileset_size, Vec2(a * 1.0, a), a);
		}
		if gs.ps.ice_skates {
			draw_sprite(cv, data::SpriteId::IceSkates, resx.tileset_size, Vec2(a * 2.0, a), a);
		}
		if gs.ps.suction_boots {
			draw_sprite(cv, data::SpriteId::SuctionBoots, resx.tileset_size, Vec2(a * 3.0, a), a);
		}

		cv.uniform.texture = resx.texdigits;
		let chips_remaining = i32::max(0, gs.field.required_chips - gs.ps.chips);
		let chips_color = if chips_remaining <= 0 { 0xFF00FFFF } else { 0xFF00FF00 };
		draw_digits(cv, chips_remaining, Vec2(ss.x as f32 - a * 1.4, a * 0.6).round(), chips_color);
		let time_remaining = if gs.field.time_limit <= 0 { -1 } else { f32::ceil((gs.field.time_limit * 60 - gs.time) as f32 / 60.0) as i32 };
		let time_color = if time_remaining <= 0 { 0xFF00FFFF } else { 0xFF00FF00 };
		draw_digits(cv, time_remaining, Vec2(ss.x as f32 - a * 1.4, a * 1.2).round(), time_color);

		{
			let tbuf = pool.get::<shade::d2::TextVertex, shade::d2::TextUniform>();
			tbuf.shader = resx.font.shader;

			let transform = Transform2f::ortho(resx.viewport.cast());
			tbuf.uniform = shade::d2::TextUniform {
				transform,
				texture: resx.font.texture,
				outline_width_absolute: 0.8,
				unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
				..Default::default()
			};
			let size = ss.y as f32 * 0.025;
			let mut scribe = shade::d2::Scribe {
				font_size: size,
				line_height: size,
				// color: Vec4(255, 255, 0, 255),
				outline: Vec4(0, 0, 0, 255),
				..Default::default()
			};
			for i in 0..gs.ps.keys.len() {
				if gs.ps.keys[i] >= 2 {
					scribe.color = match i {
						0 => Vec4(0, 255, 255, 255),
						1 => Vec4(255, 0, 0, 255),
						2 => Vec4(0, 255, 0, 255),
						3 => Vec4(255, 255, 0, 255),
						_ => Vec4(255, 255, 255, 255),
					};
					tbuf.text_box(&resx.font, &scribe, &Bounds2::c(a * i as f32, 0.0, a * (i + 1) as f32, a), shade::d2::TextAlign::BottomCenter, &format!("x{}", gs.ps.keys[i]));
				}
			}


			// let a = a * 0.75;
			scribe.font_size = a * 0.5;
			scribe.line_height = scribe.font_size * 1.2;
			scribe.color = Vec4(255, 0, 128, 255);

			// scribe.color = Vec4::unpack8(chips_color);
			// let chips_display = format!("Chips: {:0>3}", chips_remaining);
			// tbuf.text_box(&resx.font, &scribe, &Bounds2::c(ss.x as f32 - a * 3.0, 0.0, ss.x as f32, a), shade::d2::TextAlign::BottomLeft, &chips_display);
			tbuf.text_box(&resx.font, &scribe, &Bounds2::c(ss.x as f32 - a * 3.0, 0.0, ss.x as f32, a), shade::d2::TextAlign::BottomLeft, "CHIPS");

			// scribe.color = Vec4::unpack8(time_color);
			// let time_display = if time_remaining > 0 { format!("Time:  {:0>3}", time_remaining) } else { String::from("Time:  -") };
			// tbuf.text_box(&resx.font, &scribe, &Bounds2::c(ss.x as f32 - a * 3.0, a, ss.x as f32, a * 2.0), shade::d2::TextAlign::TopLeft, &time_display);
			tbuf.text_box(&resx.font, &scribe, &Bounds2::c(ss.x as f32 - a * 3.0, a, ss.x as f32, a * 2.0), shade::d2::TextAlign::TopLeft, "TIME");
		}

		let mut darken = false;
		if matches!(self.gs.ts, core::TimeState::Waiting) {
			darken = true;
			let tbuf = pool.get::<shade::d2::TextVertex, shade::d2::TextUniform>();
			tbuf.shader = resx.font.shader;

			let transform = Transform2f::ortho(resx.viewport.cast());
			tbuf.uniform = shade::d2::TextUniform {
				transform,
				texture: resx.font.texture,
				outline_width_absolute: 0.8,
				unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
				..Default::default()
			};
			let size = ss.y as f32 * 0.05;
			let mut scribe = shade::d2::Scribe {
				font_size: size,
				line_height: size,
				color: Vec4(255, 255, 255, 255),
				outline: Vec4(0, 0, 0, 255),
				..Default::default()
			};
			let level_index = format!("~ Level {} ~", self.level_number);
			let width = scribe.text_width(&mut {Vec2::ZERO}, &resx.font.font, &level_index);
			tbuf.text_write(&resx.font, &mut scribe, &mut Vec2((ss.x as f32 - width) * 0.5, ss.y as f32 * 0.75 - size * 1.2), &level_index);
			let width = scribe.text_width(&mut {Vec2::ZERO}, &resx.font.font, &self.gs.field.name);
			scribe.color = Vec4(255, 255, 0, 255);
			tbuf.text_write(&resx.font, &mut scribe, &mut Vec2((ss.x as f32 - width) * 0.5, ss.y as f32 * 0.75), &self.gs.field.name);
			if let Some(password) = &self.gs.field.password {
				let password = format!("Password: {}", password);
				let width = scribe.text_width(&mut {Vec2::ZERO}, &resx.font.font, &password);
				tbuf.text_write(&resx.font, &mut scribe, &mut Vec2((ss.x as f32 - width) * 0.5, ss.y as f32 * 0.75 + size * 1.2), &password);
			}
		}
		else if matches!(self.gs.ts, core::TimeState::Running) && self.gs.is_show_hint() {
			if let Some(hint) = &self.gs.field.hint {
				darken = true;

				let tbuf = pool.get::<shade::d2::TextVertex, shade::d2::TextUniform>();
				tbuf.shader = resx.font.shader;

				let rect = resx.viewport.cast();
				let transform = Transform2f::ortho(rect);
				tbuf.uniform = shade::d2::TextUniform {
					transform,
					texture: resx.font.texture,
					outline_width_absolute: 0.8,
					unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
					..Default::default()
				};
				let size = ss.y as f32 * 0.05;
				let scribe = shade::d2::Scribe {
					font_size: size,
					line_height: size * 1.2,
					color: Vec4(255, 255, 255, 255),
					outline: Vec4(0, 0, 0, 255),
					..Default::default()
				};
				tbuf.text_box(&resx.font, &scribe, &rect, shade::d2::TextAlign::MiddleCenter, &hint);
				// let width = scribe.text_width(&mut {Vec2::ZERO}, &resx.font, hint);
				// tbuf.text_write(&resx.font, &scribe, &mut Vec2((ss.x as f32 - width) * 0.5, ss.y as f32 * 0.5), hint);
			}
		}

		if self.darken != darken {
			self.darken = darken;
			self.darken_time = self.time;
		}

		pool.draw(g, shade::Surface::BACK_BUFFER);
	}
}

fn draw_sprite(cv: &mut shade::d2::DrawBuilder<UiVertex, UiUniform>, sprite: data::SpriteId, tex_size: Vec2<i32>, pos: Vec2<f32>, size: f32) {
	let uv = sprite.uv(tex_size);
	let tex_size = tex_size.map(|c| c as f32);
	let top_left = UiVertex { pos: Vec2f::ZERO, uv, color: [255, 255, 255, 255] };
	let bottom_left = UiVertex { pos: Vec2f::ZERO, uv: uv + Vec2(0.0, 32.0) / tex_size, color: [255, 255, 255, 255] };
	let top_right = UiVertex { pos: Vec2f::ZERO, uv: uv + Vec2(32.0, 0.0) / tex_size, color: [255, 255, 255, 255] };
	let bottom_right = UiVertex { pos: Vec2f::ZERO, uv: uv + Vec2(32.0, 32.0) / tex_size, color: [255, 255, 255, 255] };
	let sprite = shade::d2::Sprite { bottom_left, top_left, top_right, bottom_right };
	cv.sprite_rect(&sprite, &Bounds2(pos, pos + Vec2(size, size)));
}

fn draw_digits(cv: &mut shade::d2::DrawBuilder<UiVertex, UiUniform>, n: i32, pos: Vec2<f32>, color: u32) {
	if n < 0 {
		draw_digit(cv, None, pos + Vec2(0.0, 0.0), color);
		draw_digit(cv, None, pos + Vec2(17.0, 0.0), color);
		draw_digit(cv, None, pos + Vec2(34.0, 0.0), color);
	}
	else {
		let d1 = n % 10;
		let d2 = (n / 10) % 10;
		let d3 = (n / 100) % 10;

		let d3 = if d3 > 0 { Some((d3 as u8 + b'0') as char) } else { None };
		let d2 = if d2 > 0 || d3.is_some() { Some((d2 as u8 + b'0') as char) } else { None };
		let d1 = Some((d1 as u8 + b'0') as char);

		draw_digit(cv, d3, pos + Vec2(0.0, 0.0), color);
		draw_digit(cv, d2, pos + Vec2(17.0, 0.0), color);
		draw_digit(cv, d1, pos + Vec2(34.0, 0.0), color);
	}
}

fn draw_digit(cv: &mut shade::d2::DrawBuilder<UiVertex, UiUniform>, digit: Option<char>, pos: Vec2<f32>, color: u32) {
	let index = match digit {
		Some('0') => 1,
		Some('1') => 2,
		Some('2') => 3,
		Some('3') => 4,
		Some('4') => 5,
		Some('5') => 6,
		Some('6') => 7,
		Some('7') => 8,
		Some('8') => 9,
		Some('9') => 10,
		_ => 0,
	};

	let u1 = index as f32 * 17.0 / 187.0;
	let u2 = (index + 1) as f32 * 17.0 / 187.0;
	let v1 = 0.0;
	let v2 = 1.0;
	let color = Vec4::unpack8(color).into();

	let top_left = UiVertex { pos: Vec2f::ZERO, uv: Vec2(u1, v1), color };
	let bottom_left = UiVertex { pos: Vec2f::ZERO, uv: Vec2(u1, v2), color };
	let top_right = UiVertex { pos: Vec2f::ZERO, uv: Vec2(u2, v1), color };
	let bottom_right = UiVertex { pos: Vec2f::ZERO, uv: Vec2(u2, v2), color };
	let sprite = shade::d2::Sprite { bottom_left, top_left, top_right, bottom_right };
	cv.sprite_rect(&sprite, &Bounds2(pos, pos + Vec2(17.0, 25.0)));
}
