use super::*;

mod menu;
pub use self::menu::*;

mod scorecard;
pub use self::scorecard::*;

mod playtitle;
pub use self::playtitle::*;

pub fn flexv<const N: usize>(rect: Rect<f32>, gap: Option<layout::Unit>, justify: layout::Justify, template: &[layout::Unit; N]) -> [Rect<f32>; N] {
	let values = layout::flex1d(rect.mins.y, rect.maxs.y, gap, justify, template);
	let mut rects = [Rect::ZERO; N];
	for (i, &[top, bottom]) in values.iter().enumerate() {
		rects[i] = Rect::c(rect.mins.x, top, rect.maxs.x, bottom);
	}
	rects
}

pub fn flexh<const N: usize>(rect: Rect<f32>, gap: Option<layout::Unit>, justify: layout::Justify, template: &[layout::Unit; N]) -> [Rect<f32>; N] {
	let values = layout::flex1d(rect.mins.x, rect.maxs.x, gap, justify, template);
	let mut rects = [Rect::ZERO; N];
	for (i, &[left, right]) in values.iter().enumerate() {
		rects[i] = Rect::c(left, rect.mins.y, right, rect.maxs.y);
	}
	rects
}
