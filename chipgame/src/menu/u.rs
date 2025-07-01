use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct UiUniform {
	pub transform: Transform2f,
	pub texture: shade::Texture2D,
	pub color: Vec4f,
	pub gamma: f32,
}

impl Default for UiUniform {
	fn default() -> Self {
		UiUniform {
			transform: Transform2f::IDENTITY,
			texture: shade::Texture2D::INVALID,
			color: Vec4f(1.0, 1.0, 1.0, 1.0),
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
