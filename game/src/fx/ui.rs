use super::*;


#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct UiVertex {
	pub pos: Vec2f,
	pub uv: Vec2f,
	pub color: [u8; 4],
}

unsafe impl shade::TVertex for UiVertex {
	const VERTEX_LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<UiVertex>() as u16,
		alignment: mem::align_of::<UiVertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 2,
				offset: dataview::offset_of!(UiVertex.pos) as u16,
			},
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::F32,
				len: 2,
				offset: dataview::offset_of!(UiVertex.uv) as u16,
			},
			shade::VertexAttribute {
				format: shade::VertexAttributeFormat::U8Norm,
				len: 4,
				offset: dataview::offset_of!(UiVertex.color) as u16,
			},
		],
	};
}

impl shade::d2::ToVertex<UiVertex> for UiVertex {
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> UiVertex {
		UiVertex { pos, ..*self }
	}
}

#[derive(Copy, Clone, Debug, dataview::Pod)]
#[repr(C)]
pub struct UiUniforms {
	pub transform: Transform2f,
	pub texture: shade::Texture2D,
	pub color: [f32; 4],
}

impl Default for UiUniforms {
	fn default() -> Self {
		UiUniforms {
			transform: cvmath::Transform2f::IDENTITY,
			texture: shade::Texture2D::INVALID,
			color: [1.0, 1.0, 1.0, 1.0],
		}
	}
}

unsafe impl shade::TUniform for UiUniforms {
	const UNIFORM_LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: mem::size_of::<UiUniforms>() as u16,
		alignment: mem::align_of::<UiUniforms>() as u16,
		attributes: &[
			shade::UniformAttribute {
				name: "transform",
				ty: shade::UniformType::Mat3x2 { order: shade::UniformMatOrder::RowMajor },
				offset: dataview::offset_of!(UiUniforms.transform) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "tex",
				ty: shade::UniformType::Sampler2D(0),
				offset: dataview::offset_of!(UiUniforms.texture) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "color",
				ty: shade::UniformType::F4,
				offset: dataview::offset_of!(UiUniforms.color) as u16,
				len: 1,
			},
		],
	};
}

fn foo(from: Rect<f32>, to: Rect<f32>) -> Transform2<f32> {
	let sx = (to.maxs.x - to.mins.x) / (from.maxs.x - from.mins.x);
	let sy = (to.maxs.y - to.mins.y) / (from.maxs.y - from.mins.y);
	Transform2 {
		a11: sx, a12: 0.0, a13: to.mins.x - from.mins.x * sx,
		a21: 0.0, a22: sy, a23: to.mins.y - from.mins.y * sy,
	}
}

impl FxState {
	pub fn render_ui(&self, g: &mut shade::Graphics) {
		g.begin().unwrap();

		let ss = self.resources.screen_size;

		let mut cv = shade::d2::CommandBuffer::<UiVertex, UiUniforms>::new();

		let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));
		// let transform = Transform2f::IDENTITY;

		cv.push_uniform(ui::UiUniforms {
			transform,//: Transform2f::IDENTITY,
			texture: self.resources.texdigits,
			color: [1.0, 1.0, 1.0, 1.0],
		});

		cv.shader = self.resources.uishader;
		cv.blend_mode = shade::BlendMode::Alpha;
		cv.viewport = Bounds::vec(ss);

		let paint = shade::d2::Paint {
			template: UiVertex {
				pos: Vec2f::ZERO,
				uv: Vec2f::ZERO,
				color: [255, 255, 255, 128],
			},
		};
		let a = 32.0;
		// cv.fill_rect(&paint, &Rect::c(-1.0, -1.0 + 64.0 / ss.y as f32 * 2.0, 1.0, -1.0));
		// cv.fill_rect(&paint, &Rect(transform * Vec2(0.0, ss.y as f32 - 64.0), transform * Vec2(ss.x as f32, ss.y as f32)));
		// cv.fill_rect(&paint, &Rect::c(0.0, ss.y as f32 - 64.0, ss.x as f32, ss.y as f32));
		// cv.fill_rect(&paint, &Rect::c(ss.x as f32 - 200.0, 0.0, ss.x as f32, ss.y as f32));
		cv.fill_rect(&paint, &Rect::c(0.0, 0.0, a * 4.0, a * 2.0));

		cv.push_uniform_f(|u| {
			ui::UiUniforms {
				texture: self.resources.tileset,
				..*u
			}
		});

		let ref gs = self.gs;

		if gs.ps.keys[0] > 0 {
			draw_sprite(&mut cv, Sprite::BlueKey, self.resources.tileset_size, Vec2(a * 0.0, 0.0));
		}
		if gs.ps.keys[1] > 0 {
			draw_sprite(&mut cv, Sprite::RedKey, self.resources.tileset_size, Vec2(a * 1.0, 0.0));
		}
		if gs.ps.keys[2] > 0 {
			draw_sprite(&mut cv, Sprite::GreenKey, self.resources.tileset_size, Vec2(a * 2.0, 0.0));
		}
		if gs.ps.keys[3] > 0 {
			draw_sprite(&mut cv, Sprite::YellowKey, self.resources.tileset_size, Vec2(a * 3.0, 0.0));
		}

		if gs.ps.flippers {
			draw_sprite(&mut cv, Sprite::PowerFlippers, self.resources.tileset_size, Vec2(a * 0.0, a));
		}
		if gs.ps.fire_boots {
			draw_sprite(&mut cv, Sprite::PowerFireBoots, self.resources.tileset_size, Vec2(a * 1.0, a));
		}
		if gs.ps.ice_skates {
			draw_sprite(&mut cv, Sprite::PowerIceSkates, self.resources.tileset_size, Vec2(a * 2.0, a));
		}
		if gs.ps.suction_boots {
			draw_sprite(&mut cv, Sprite::PowerSuctionBoots, self.resources.tileset_size, Vec2(a * 3.0, a));
		}

		cv.push_uniform_f(|u| {
			ui::UiUniforms {
				texture: self.resources.texdigits,
				..*u
			}
		});
		let chips_remaining = i32::max(0, gs.field.chips - gs.ps.chips);
		let color = if chips_remaining <= 0 { 0xFF00FFFF } else { 0xFF00FF00 };
		draw_digits(&mut cv, chips_remaining, Vec2(a * 4.0, a * 2.0), color);
		let time_remaining = if gs.field.time <= 0 { -1 } else { f32::ceil((gs.field.time * 60 - gs.time) as f32 / 60.0) as i32 };
		let color = if time_remaining <= 0 { 0xFF00FFFF } else { 0xFF00FF00 };
		draw_digits(&mut cv, time_remaining, Vec2(a * 4.0, a * 2.0 + 25.0), color);

		cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();

		if matches!(self.gs.ts, core::TimeState::Waiting) {
			if let Some(hint) = &self.gs.field.hint {
				if let Some(font) = &self.resources.font {
					let mut tbuf = shade::d2::TextBuffer::new();
					tbuf.shader = self.resources.fontshader;
					tbuf.viewport = cvmath::Rect::vec(ss);

					let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));
					tbuf.push_uniform(shade::d2::TextUniform {
						transform,
						texture: self.resources.fonttexture,
						outline_width_absolute: 0.8,
						unit_range: Vec2::dup(4.0f32) / Vec2(232.0f32, 232.0f32),
						..Default::default()
					});
					let scribe = shade::d2::Scribe {
						font_size: 32.0,
						font_width_scale: 1.0,
						line_height: 32.0,
						baseline: 0.0,
						x_pos: 0.0,
						letter_spacing: 0.0,
						top_skew: 0.0,
						color: Vec4(255, 255, 255, 255),
						outline: Vec4(0, 0, 0, 255),
					};
					tbuf.text_write(&scribe, font, &mut Vec2(0.0, 0.0), hint);
					tbuf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
				}
			}
		}

		g.end().unwrap();
	}
}

fn draw_sprite(cv: &mut shade::d2::CommandBuffer<UiVertex, UiUniforms>, sprite: Sprite, tex_size: Vec2<i32>, pos: Vec2<f32>) {
	let a = 32.0;
	let uv = sprite.uv(tex_size);
	let tex_size = tex_size.map(|c| c as f32);
	let top_left = UiVertex { pos: Vec2f::ZERO, uv, color: [255, 255, 255, 255] };
	let bottom_left = UiVertex { pos: Vec2f::ZERO, uv: uv + Vec2(0.0, 32.0) / tex_size, color: [255, 255, 255, 255] };
	let top_right = UiVertex { pos: Vec2f::ZERO, uv: uv + Vec2(32.0, 0.0) / tex_size, color: [255, 255, 255, 255] };
	let bottom_right = UiVertex { pos: Vec2f::ZERO, uv: uv + Vec2(32.0, 32.0) / tex_size, color: [255, 255, 255, 255] };
	let stamp = shade::d2::Stamp { bottom_left, top_left, top_right, bottom_right };
	cv.stamp_rect(&stamp, &Rect(pos, pos + Vec2(a, a)));
}

fn draw_digits(cv: &mut shade::d2::CommandBuffer<UiVertex, UiUniforms>, n: i32, pos: Vec2<f32>, color: u32) {
	if n < 0 {
		draw_digit(cv, None, pos + (0.0, 0.0), color);
		draw_digit(cv, None, pos + (17.0, 0.0), color);
		draw_digit(cv, None, pos + (34.0, 0.0), color);
	}
	else {
		let d1 = n % 10;
		let d2 = (n / 10) % 10;
		let d3 = (n / 100) % 10;

		let d3 = if d3 > 0 { Some((d3 as u8 + b'0') as char) } else { None };
		let d2 = if d2 > 0 || d3.is_some() { Some((d2 as u8 + b'0') as char) } else { None };
		let d1 = Some((d1 as u8 + b'0') as char);

		draw_digit(cv, d3, pos + (0.0, 0.0), color);
		draw_digit(cv, d2, pos + (17.0, 0.0), color);
		draw_digit(cv, d1, pos + (34.0, 0.0), color);
	}
}

fn draw_digit(cv: &mut shade::d2::CommandBuffer<UiVertex, UiUniforms>, digit: Option<char>, pos: Vec2<f32>, color: u32) {
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
	let stamp = shade::d2::Stamp { bottom_left, top_left, top_right, bottom_right };

	cv.stamp_rect(&stamp, &Rect(pos, pos + Vec2(17.0, 25.0)));
}
