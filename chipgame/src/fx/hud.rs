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

		// Draw the backgrounds for various UI elements
		let paint = shade::d2::Paint {
			template: UiVertex {
				pos: Vec2f::ZERO,
				uv: Vec2f::ZERO,
				color: [255, 255, 255, 64],
			},
		};
		let ss = resx.viewport.size();
		let a = f32::max(ss.y as f32 * 0.075, ss.x as f32 * 0.05);
		let y = resx.viewport.bottom() as f32 - a * 1.1;
		let ref gs = self.gs;

		let [[keys_x1, keys_x2], [items_x1, items_x2]] = shade::d2::layout::flex1d(resx.viewport.left() as f32, resx.viewport.right() as f32, None, shade::d2::layout::Justify::SpaceAround, &[
			shade::d2::layout::Unit::Abs(a * 5.0),
			shade::d2::layout::Unit::Abs(a * 5.0),
		]);
		if gs.ps.keys.iter().any(|&k| k > 0) {
			cv.fill_rect(&paint, &Bounds2::c(keys_x1, y, keys_x2, resx.viewport.bottom() as f32));
		}
		if gs.ps.flippers || gs.ps.fire_boots || gs.ps.ice_skates || gs.ps.suction_boots {
			cv.fill_rect(&paint, &Bounds2::c(items_x1, y, items_x2, resx.viewport.bottom() as f32));
		}
		if !self.darken {
			cv.fill_rect(&paint, &Bounds2::c(keys_x1, 0.0, keys_x2, a * 0.9));
			cv.fill_rect(&paint, &Bounds2::c(items_x1, 0.0, items_x2, a * 0.9));
		}

		// Draw the inventory items
		cv.uniform.texture = resx.tileset;
		if gs.ps.keys[0] > 0 {
			draw_sprite(cv, data::SpriteId::BlueKey, resx.tileset_size, Vec2(keys_x1 + a * 0.5, y), a);
		}
		if gs.ps.keys[1] > 0 {
			draw_sprite(cv, data::SpriteId::RedKey, resx.tileset_size, Vec2(keys_x1 + a * 1.5, y), a);
		}
		if gs.ps.keys[2] > 0 {
			draw_sprite(cv, data::SpriteId::GreenKey, resx.tileset_size, Vec2(keys_x1 + a * 2.5, y), a);
		}
		if gs.ps.keys[3] > 0 {
			draw_sprite(cv, data::SpriteId::YellowKey, resx.tileset_size, Vec2(keys_x1 + a * 3.5, y), a);
		}
		if gs.ps.flippers {
			draw_sprite(cv, data::SpriteId::Flippers, resx.tileset_size, Vec2(items_x1 + a * 0.5, y), a);
		}
		if gs.ps.fire_boots {
			draw_sprite(cv, data::SpriteId::FireBoots, resx.tileset_size, Vec2(items_x1 + a * 1.5, y), a);
		}
		if gs.ps.ice_skates {
			draw_sprite(cv, data::SpriteId::IceSkates, resx.tileset_size, Vec2(items_x1 + a * 2.5, y), a);
		}
		if gs.ps.suction_boots {
			draw_sprite(cv, data::SpriteId::SuctionBoots, resx.tileset_size, Vec2(items_x1 + a * 3.5, y), a);
		}

		// Draw the CHIPS and TIME counters
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
					tbuf.text_box(&resx.font, &scribe, &Bounds2::c(keys_x1 + a * (i as f32 + 0.5), y + 0.0, keys_x1 + a * (i as f32 + 1.5), y + a), shade::d2::TextAlign::BottomCenter, &format!("x{}", gs.ps.keys[i]));
				}
			}

			scribe.font_size = a * 0.5;
			scribe.line_height = scribe.font_size * 1.2;

			let digits_w = scribe.text_width(&mut Vec2(0.0, 0.0), &resx.font.font, "000");
			let chips_x = (keys_x1 + keys_x2) * 0.5;
			let chips_remaining = i32::max(0, gs.field.required_chips - gs.ps.chips);
			let time_x = (items_x1 + items_x2) * 0.5;
			let time_remaining = if gs.field.time_limit <= 0 { -1 } else { f32::ceil((gs.field.time_limit * 60 - gs.time) as f32 / 60.0) as i32 };

			scribe.color = Vec4(255, 0, 128, 255);
			tbuf.text_box(&resx.font, &scribe, &Bounds2::c(chips_x, 0.0, chips_x, a), shade::d2::TextAlign::TopRight, "CHIPS:");
			scribe.color = if chips_remaining <= 0 { Vec4::unpack8(0xFF00FFFF) } else { Vec4::unpack8(0xFF00FF00) };
			tbuf.text_box(&resx.font, &scribe, &Bounds2::c(chips_x + a * 0.125, 0.0, chips_x, a), shade::d2::TextAlign::TopLeft, &format!("{chips_remaining}"));

			scribe.color = Vec4(255, 0, 128, 255);
			tbuf.text_box(&resx.font, &scribe, &Bounds2::c(time_x, 0.0, time_x, a), shade::d2::TextAlign::TopRight, "TIME:");
			scribe.color = if time_remaining <= 0 { Vec4::unpack8(0xFF00FFFF) } else { Vec4::unpack8(0xFF00FF00) };
			if time_remaining >= 0 {
				tbuf.text_box(&resx.font, &scribe, &Bounds2::c(time_x, 0.0, time_x + digits_w + a * 0.125, a), shade::d2::TextAlign::TopRight, &format!("{time_remaining}"));
			}
			else {
				tbuf.text_box(&resx.font, &scribe, &Bounds2::c(time_x + a * 0.125, 0.0, time_x, a), shade::d2::TextAlign::TopLeft, "- - -");
			}
		}

		// Draw the level title or hint text
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
