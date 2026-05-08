use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EffectType {
	Splash,
	Sparkles,
	Fireworks,
}

#[derive(Clone)]
pub struct Effect {
	pub ty: EffectType,
	pub pos: Vec3f,
	pub start: f64,
}

impl Effect {
	pub fn draw(&self, cv: &mut shade::im::DrawBuilder<Vertex, Uniform>, time: f64) {
		let mut p = cv.begin(shade::PrimType::Triangles, 4, 2);
		let t = f32::clamp((time - self.start) as f32, 0.0, 1.0);
		// 12 frames of animation
		let aindex = f32::floor(t * 13.0).min(12.0);
		let d_size = 96.0;
		let u = aindex * d_size;
		let v = match self.ty {
			EffectType::Splash => d_size * 0.0,
			EffectType::Sparkles => d_size * 1.0,
			EffectType::Fireworks => d_size * 2.0,
		};
		p.add_indices_quad();
		let s = 32.0;
		let color = [255; 4];
		p.add_vertices(&[
			Vertex { pos: self.pos + Vec3f(-s, s, 1.0), uv: Vec2f(u, v), color },
			Vertex { pos: self.pos + Vec3f(-s, -s, 1.0), uv: Vec2f(u, v + 96.0), color },
			Vertex { pos: self.pos + Vec3f(s, -s, 1.0), uv: Vec2f(u + 96.0, v + 96.0), color },
			Vertex { pos: self.pos + Vec3f(s, s, 1.0), uv: Vec2f(u + 96.0, v), color },
		]);
	}
}
