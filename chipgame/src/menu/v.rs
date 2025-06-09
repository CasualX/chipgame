use super::*;

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct UiVertex {
	pub pos: Vec2f,
	pub uv: Vec2f,
	pub color: [u8; 4],
}

unsafe impl shade::TVertex for UiVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<UiVertex>() as u16,
		alignment: mem::align_of::<UiVertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "a_pos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(UiVertex.pos) as u16,
			},
			shade::VertexAttribute {
				name: "a_texcoord",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(UiVertex.uv) as u16,
			},
			shade::VertexAttribute {
				name: "a_color",
				format: shade::VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(UiVertex.color) as u16,
			},
		],
	};
}

impl shade::d2::ToVertex<UiVertex> for UiVertex {
	fn to_vertex(&self, pos: Vec2f, _index: usize) -> UiVertex {
		UiVertex { pos, ..*self }
	}
}
