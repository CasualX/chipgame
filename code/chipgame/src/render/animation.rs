use super::*;

#[derive(Clone, Debug)]
pub struct MoveStep {
	pub start_pos: Vec3f,
	pub end_pos: Vec3f,
	pub move_time: f64,
	pub duration: f32,
	pub jump_height: f32,
	pub sprite_after: Option<chipty::SpriteId>,
}

impl MoveStep {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		if self.duration <= 0.0 {
			obj.pos = self.end_pos;
			return false;
		}
		let elapsed = (ctx.time - self.move_time) as f32;
		let fraction = (elapsed / self.duration).clamp(0.0, 1.0);
		let eased = 1.0 - (1.0 - fraction).powf(4.0);
		obj.pos = self.start_pos.lerp(self.end_pos, eased);
		obj.pos.z += self.jump_height * (4.0 * fraction * (1.0 - fraction));
		if fraction < 1.0 {
			return true;
		}
		if let Some(sprite) = self.sprite_after {
			obj.sprite = sprite;
		}
		return false;
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

/// Infinite sprite animation loop.
#[derive(Clone, Debug)]
pub struct SpriteAnimLoop {
	pub start_time: f64,
	pub frame_rate: f32,
}

impl SpriteAnimLoop {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		let elapsed = (ctx.time - self.start_time) as f32;
		obj.frame = (elapsed * self.frame_rate) as i32 as u16;
		true
	}
}

/// Finite sprite animation sequence.
#[derive(Clone, Debug)]
pub struct SpriteAnimSeq {
	pub start_time: f64,
	pub frame_count: u16,
	pub frame_rate: f32,
}

impl SpriteAnimSeq {
	pub fn animate(&self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		let elapsed = (ctx.time - self.start_time) as f32;
		let total_duration = self.frame_count as f32 / self.frame_rate;
		if elapsed >= total_duration {
			obj.frame = self.frame_count - 1;
			return false;
		}
		let frame = (elapsed * self.frame_rate) as i32 as u16;
		obj.frame = frame.min(self.frame_count - 1);
		return true;
	}
}

#[derive(Clone, Debug)]
pub enum AnimState {
	MoveStep(MoveStep),
	MoveVel(MoveVel),
	MoveZ(MoveZ),
	FadeOut(FadeOut),
	FadeTo(FadeTo),
	AnimLoop(SpriteAnimLoop),
	AnimSeq(SpriteAnimSeq),
}

impl AnimState {
	pub fn animate(&mut self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		match self {
			AnimState::MoveStep(step) => step.animate(obj, ctx),
			AnimState::MoveVel(vel) => vel.animate(obj, ctx),
			AnimState::MoveZ(mz) => mz.animate(obj, ctx),
			AnimState::FadeOut(fade) => fade.animate(obj, ctx),
			AnimState::FadeTo(fade) => fade.animate(obj, ctx),
			AnimState::AnimSeq(anim) => anim.animate(obj, ctx),
			AnimState::AnimLoop(anim) => anim.animate(obj, ctx),
		}
	}
}
