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

	pub blend: f32,
}

impl Camera {
	pub fn setup(&self, size: Vec2i) -> shade::d3::CameraSetup {
		let position = self.target + self.eye_offset;
		let view = {
			let target = self.target_fast;
			let up = Vec3(0.0, -1.0, 0.0);
			Transform3f::look_at(position, target, up, Hand::LH)
		};
		let aspect_ratio = size.x as f32 / size.y as f32;
		let fov_y = Angle::deg(90.0);
		let near = 10.0;
		let far = 2000.0;
		// let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (Hand::LH, Clip::NO));
		// let half_height = 200.0;
		// let projection = Mat4::ortho(-half_height * aspect_ratio, half_height * aspect_ratio, -half_height, half_height, near, far, (Hand::LH, Clip::NO));
		let projection = Mat4::blend_ortho_perspective(self.blend, position.distance(self.target_fast), fov_y, aspect_ratio, near, far, (Hand::LH, Clip::NO));
		let view_proj = projection * view;
		shade::d3::CameraSetup {
			surface: shade::Surface::BACK_BUFFER,
			viewport: Bounds2::vec(size),
			aspect_ratio,
			position,
			view,
			near,
			far,
			projection,
			view_proj,
			inv_view_proj: view_proj.inverse(),
			clip: Clip::NO,
		}
	}
}

impl FxState {
	pub fn set_game_camera(&mut self, time: f32) {
		self.camera.blend = f32::clamp((time - 0.0) * 0.5, 0.0, 1.0);

		let ent_pos = if let Some(obj) = self.camera.object_h.and_then(|h| self.objects.get(h)) {
			self.camera.eye_offset = Vec3::new(0.0, 1.0 * 32.0 * (self.camera.blend.max(0.001) * 1.75).min(1.0), 200.0);
			obj.lerp_pos + (16.0, 32.0 * 1.5, 0.0)
		}
		else {
			self.camera.eye_offset = Vec3::new(0.0, 1.0 * 32.0, 400.0);
			self.camera.target
		};

		self.camera.target_fast = self.camera.target_fast.exp_decay(ent_pos, 25.0, 1.0 / 60.0);
		self.camera.target = self.camera.target.exp_decay(ent_pos, 15.0, 1.0 / 60.0).set_x(self.camera.target_fast.x);
	}
}
