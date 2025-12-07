use super::*;

#[derive(Clone, Debug)]
pub struct Animation {
	pub anims: Vec<AnimState>,
	pub unalive_after_anim: bool,
}

impl Animation {
	pub fn update(&mut self, obj: &mut ObjectData, ctx: &UpdateCtx) -> bool {
		self.anims.retain_mut(|anim| anim.animate(obj, ctx));
		!(self.unalive_after_anim && self.anims.is_empty())
	}
}

#[derive(Clone, Debug)]
pub struct ObjectData {
	pub pos: Vec3<f32>,
	pub sprite: chipty::SpriteId,
	pub frame: u16,
	pub model: chipty::ModelId,
	pub alpha: f32,
	pub visible: bool,
	pub greyscale: bool,
}

#[derive(Clone, Debug)]
pub struct Object {
	pub data: ObjectData,
	pub anim: Animation,
}

impl Object {
	pub fn update(&mut self, ctx: &UpdateCtx) -> bool {
		self.anim.update(&mut self.data, ctx)
	}
}
