use super::*;

fn foo(from: Rect<f32>, to: Rect<f32>) -> Transform2<f32> {
	let sx = (to.maxs.x - to.mins.x) / (from.maxs.x - from.mins.x);
	let sy = (to.maxs.y - to.mins.y) / (from.maxs.y - from.mins.y);
	Transform2 {
		a11: sx, a12: 0.0, a13: to.mins.x - from.mins.x * sx,
		a21: 0.0, a22: sy, a23: to.mins.y - from.mins.y * sy,
	}
}

#[derive(Default)]
pub struct MainMenu {
	pub selected: u8,
}

impl MainMenu {
	const ITEMS: [&'static str; 7] = ["New game", "Continue", "Go to level", "High scores", "Options", "About", "Exit"];
	pub fn update(&mut self, input: &core::Input, old: &core::Input) {
		if input.up && !old.up {
			self.selected = if self.selected > 0 { self.selected - 1 } else { self.selected };
		}
		if input.down && !old.down {
			self.selected = if self.selected < Self::ITEMS.len() as u8 - 1 { self.selected + 1 } else { self.selected };
		}
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
