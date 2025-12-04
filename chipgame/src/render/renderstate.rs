use super::*;

#[derive(Default)]
pub struct RenderField {
	pub width: i32,
	pub height: i32,
	pub terrain: Vec<chipty::Terrain>,
}

impl RenderField {
	pub fn get_terrain(&self, pos: Vec2i) -> chipty::Terrain {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return chipty::Terrain::Blank;
		}
		let index = (y * self.width + x) as usize;
		self.terrain.get(index).cloned().unwrap_or(chipty::Terrain::Blank)
	}
	pub fn set_terrain(&mut self, pos: Vec2i, terrain: chipty::Terrain) {
		let Vec2i { x, y } = pos;
		if x < 0 || y < 0 || x >= self.width || y >= self.height {
			return;
		}
		let index = (y * self.width + x) as usize;
		if let Some(ptr) = self.terrain.get_mut(index) {
			*ptr = terrain;
		}
	}
}

pub struct UpdateCtx {
	pub time: f64,
	pub dt: f64,
}

#[derive(Default)]
pub struct RenderState {
	pub objects: ObjectMap,
	pub field: RenderField,
	pub effects: Vec<Effect>,
	pub tiles: &'static [TileGfx],
}

impl RenderState {
	pub fn clear(&mut self) {
		self.objects.clear();
		self.field.width = 0;
		self.field.height = 0;
		self.field.terrain.clear();
		self.effects.clear();
	}
	pub fn update(&mut self, ctx: &UpdateCtx) {
		self.objects.retain(|_, obj| obj.update(ctx));
		self.effects.retain(|efx| ctx.time < efx.start + 1.0);
	}
	pub fn draw(&self, g: &mut shade::Graphics, resx: &Resources, camera: &shade::d3::CameraSetup, time: f64) {
		{
			let mut cv = shade::d2::DrawBuilder::<render::Vertex, render::Uniform>::new();
			cv.viewport = resx.viewport;
			cv.depth_test = Some(shade::DepthTest::LessEqual);
			cv.cull_mode = Some(shade::CullMode::CW);
			cv.shader = resx.shader;
			cv.uniform.transform = camera.view_proj;
			cv.uniform.texture = resx.spritesheet_texture;
			cv.uniform.pixel_bias = resx.pixel_art_bias;
			render::field(&mut cv, self, resx, time, 1.0);
			cv.draw(g, shade::Surface::BACK_BUFFER);
		}

		// Render the effects
		{
			let mut cv = shade::d2::DrawBuilder::<Vertex, Uniform>::new();
			cv.viewport = resx.viewport;
			cv.blend_mode = shade::BlendMode::Solid;
			cv.depth_test = Some(shade::DepthTest::Always);
			// cv.cull_mode = Some(shade::CullMode::CW);

			cv.shader = resx.shader;
			cv.uniform.transform = camera.view_proj;
			cv.uniform.texture = resx.effects;

			for efx in &self.effects {
				efx.draw(&mut cv, time);
			}
			cv.draw(g, shade::Surface::BACK_BUFFER);
		}
	}
}
