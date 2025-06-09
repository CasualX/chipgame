use super::*;

#[derive(Copy, Clone, Debug, dataview::Pod)]
#[repr(C)]
pub struct UiUniform {
	pub transform: Transform2f,
	pub texture: shade::Texture2D,
	pub color: [f32; 4],
	pub gamma: f32,
}

impl Default for UiUniform {
	fn default() -> Self {
		UiUniform {
			transform: Transform2f::IDENTITY,
			texture: shade::Texture2D::INVALID,
			color: [1.0, 1.0, 1.0, 1.0],
			gamma: 2.2,
		}
	}
}

impl shade::UniformVisitor for UiUniform {
	fn visit(&self, set: &mut dyn shade::UniformSetter) {
		set.value("u_transform", &self.transform);
		set.value("u_texture", &self.texture);
		set.value("u_color", &self.color);
		set.value("u_gamma", &self.gamma);
	}
}
