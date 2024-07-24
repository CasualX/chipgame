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
			transform: cvmath::Transform2f::IDENTITY,
			texture: shade::Texture2D::INVALID,
			color: [1.0, 1.0, 1.0, 1.0],
			gamma: 2.2,
		}
	}
}

unsafe impl shade::TUniform for UiUniform {
	const UNIFORM_LAYOUT: &'static shade::UniformLayout = &shade::UniformLayout {
		size: mem::size_of::<UiUniform>() as u16,
		alignment: mem::align_of::<UiUniform>() as u16,
		attributes: &[
			shade::UniformAttribute {
				name: "u_transform",
				ty: shade::UniformType::Mat3x2 { order: shade::UniformMatOrder::RowMajor },
				offset: dataview::offset_of!(UiUniform.transform) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "u_texture",
				ty: shade::UniformType::Sampler2D(0),
				offset: dataview::offset_of!(UiUniform.texture) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "u_color",
				ty: shade::UniformType::F4,
				offset: dataview::offset_of!(UiUniform.color) as u16,
				len: 1,
			},
			shade::UniformAttribute {
				name: "u_gamma",
				ty: shade::UniformType::F1,
				offset: dataview::offset_of!(UiUniform.gamma) as u16,
				len: 1,
			},
		],
	};
}
