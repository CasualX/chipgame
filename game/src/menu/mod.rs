use std::mem;
use cvmath::*;
use crate::core;
use crate::fx::Resources;

mod event;
mod main;
mod gamewin;
mod pausemenu;
mod u;
mod v;

pub use self::event::*;
pub use self::main::*;
pub use self::gamewin::*;
pub use self::pausemenu::*;
pub use self::u::*;
pub use self::v::*;

fn foo(from: Rect<f32>, to: Rect<f32>) -> Transform2<f32> {
	let sx = (to.maxs.x - to.mins.x) / (from.maxs.x - from.mins.x);
	let sy = (to.maxs.y - to.mins.y) / (from.maxs.y - from.mins.y);
	Transform2 {
		a11: sx, a12: 0.0, a13: to.mins.x - from.mins.x * sx,
		a21: 0.0, a22: sy, a23: to.mins.y - from.mins.y * sy,
	}
}

fn darken(g: &mut shade::Graphics, resx: &Resources, alpha: u8) {
	let mut cv = shade::d2::CommandBuffer::<UiVertex, UiUniform>::new();

	cv.blend_mode = shade::BlendMode::Alpha;
	cv.shader = resx.colorshader;
	cv.viewport = cvmath::Rect::vec(resx.screen_size);

	let paint = shade::d2::Paint {
		template: UiVertex { pos: Vec2::ZERO, uv: Vec2::ZERO, color: [0, 0, 0, alpha] },
	};
	cv.fill_rect(&paint, &cvmath::Rect::c(-1.0, 1.0, 1.0, -1.0));

	cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
}
