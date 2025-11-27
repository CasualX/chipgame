use super::*;

#[derive(Clone, Debug)]
pub struct MoveStep {
	pub dest: Vec2<i32>,
	pub move_time: f64,
	pub move_spd: f32,
}

impl MoveStep {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		let t = f32::min(1.0, (ctx.time - self.move_time) as f32 / self.move_spd);
		let dest = self.dest.map(|c| c as f32 * 32.0).vec3(0.0);
		obj.pos = obj.pos.exp_decay(dest, 100.0 * self.move_spd, ctx.dt as f32);
		if t == 1.0 {
			obj.pos = dest;
			return false;
		}
		return true;
	}
}

#[derive(Clone, Debug)]
pub struct MoveVel {
	pub vel: Vec3<f32>,
}

impl MoveVel {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		obj.pos += self.vel * ctx.dt as f32;
		return true;
	}
}

#[derive(Clone, Debug)]
pub struct FadeOut {
	pub atime: f64,
}

impl FadeOut {
	pub fn animate(&mut self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		if self.atime == 0.0 {
			self.atime = ctx.time;
		}
		obj.alpha = f32::max(0.0, 1.0 - (ctx.time - self.atime) as f32 * 4.0);
		return obj.alpha > 0.0;
	}
}

#[derive(Clone, Debug)]
pub struct FadeIn {
	pub atime: f64,
}

impl FadeIn {
	pub fn animate(&mut self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		if self.atime == 0.0 {
			self.atime = ctx.time;
		}
		obj.alpha = f32::min(1.0, (ctx.time - self.atime) as f32 * 8.0);
		return obj.alpha < 1.0;
	}
}

#[derive(Clone, Debug)]
pub struct FadeTo {
	pub target_alpha: f32,
	pub fade_spd: f32,
}

impl FadeTo {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		if obj.alpha < self.target_alpha {
			obj.alpha = f32::min(self.target_alpha, obj.alpha + self.fade_spd * ctx.dt as f32);
		}
		else {
			obj.alpha = f32::max(self.target_alpha, obj.alpha - self.fade_spd * ctx.dt as f32);
		}
		return obj.alpha != self.target_alpha;
	}
}

#[derive(Clone, Debug)]
pub struct MoveZ {
	pub target_z: f32,
	pub move_spd: f32,
}

impl MoveZ {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		if obj.pos.z < self.target_z {
			obj.pos.z = f32::min(self.target_z, obj.pos.z + self.move_spd * ctx.dt as f32);
		}
		else {
			obj.pos.z = f32::max(self.target_z, obj.pos.z - self.move_spd * ctx.dt as f32);
		}
		return obj.pos.z != self.target_z;
	}
}

#[derive(Clone, Debug)]
pub enum AnimState {
	MoveStep(MoveStep),
	MoveVel(MoveVel),
	MoveZ(MoveZ),
	FadeOut(FadeOut),
	FadeIn(FadeIn),
	FadeTo(FadeTo),
}

impl AnimState {
	pub fn animate(&mut self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		match self {
			AnimState::MoveStep(step) => step.animate(obj, ctx),
			AnimState::MoveVel(vel) => vel.animate(obj, ctx),
			AnimState::MoveZ(mz) => mz.animate(obj, ctx),
			AnimState::FadeOut(fade) => fade.animate(obj, ctx),
			AnimState::FadeIn(fade) => fade.animate(obj, ctx),
			AnimState::FadeTo(fade) => fade.animate(obj, ctx),
		}
	}
}
