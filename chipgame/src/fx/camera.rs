use super::*;

#[derive(Default)]
pub struct Camera {
	// Object to follow with the camera
	pub object_h: Option<ObjectHandle>,

	/// Look at target
	pub target: Vec3<f32>,
	pub target_fast: Vec3<f32>,
	/// Eye offset from the target
	pub eye_offset: Vec3<f32>,
}

impl Camera {
	pub fn setup(&self, size: Vec2i) -> shade::d3::CameraSetup {
		let aspect_ratio = size.x as f32 / size.y as f32;
		let projection = Mat4::perspective_fov(Deg(45.0), size.x as f32, size.y as f32, 10.0, 2000.0, (Hand::LH, Clip::NO));
		// let projection = Mat4::ortho(-200.0 * aspect_ratio, 200.0 * aspect_ratio, -200.0, 200.0, 0.1, 2000.0, (Hand::LH, Clip::NO));
		let view = {
			let eye = self.target + self.eye_offset;
			let target = self.target_fast;
			let up = Vec3(0.0, -1.0, 0.0);
			Mat4::look_at(eye, target, up, Hand::LH)
		};
		let view_proj = projection * view;
		shade::d3::CameraSetup {
			surface: shade::Surface::BACK_BUFFER,
			viewport: Bounds2::vec(size),
			aspect_ratio,
			position: self.target + self.eye_offset,
			near: 0.1,
			far: 2000.0,
			projection,
			view,
			view_proj,
			inv_view_proj: view_proj.inverse(),
			clip: Clip::NO,
		}
	}
}

impl FxState {
	pub fn set_game_camera(&mut self) {
		let ent_pos = if let Some(obj) = self.camera.object_h.and_then(|h| self.objects.get(h)) {
			self.camera.eye_offset = Vec3::new(0.0, 8.0 * 32.0, 400.0);
			obj.lerp_pos + (16.0, 32.0 * 1.5, 0.0)
		}
		else {
			self.camera.eye_offset = Vec3::new(0.0, 2.0 * 32.0, 800.0);
			self.camera.target
		};

		self.camera.target_fast = self.camera.target_fast.exp_decay(ent_pos, 25.0, 1.0 / 60.0);
		self.camera.target = self.camera.target.exp_decay(ent_pos, 15.0, 1.0 / 60.0).set_x(self.camera.target_fast.x);
	}
}
