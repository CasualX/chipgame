use super::*;

#[derive(Default)]
pub struct Camera {
	// Object to follow with the camera
	pub object: Option<ObjectHandle>,
	// Camera offset from the object
	pub offset: Vec3f,

	// Look at target (snaps to grid)
	pub target: Vec3f,
	// Look at target (slowly tracks the target)
	pub target_slow: Vec3f,
	// Look at target (quickly tracks the target)
	pub target_fast: Vec3f,

	// Blend between orthographic and perspective projection
	pub blend: f32,
	pub perspective: bool,
}

impl Camera {
	pub fn setup(&self, size: Vec2i) -> shade::d3::CameraSetup {
		let offset_y = self.offset.y * self.blend;
		let position = self.target_slow + self.offset.set_y(offset_y);
		let view = Transform3f::look_at(position, self.target_fast, -Vec3f::Y, Hand::LH);
		let aspect_ratio = size.x as f32 / size.y as f32;
		let fov_y = Angle::deg(90.0);
		let near = 10.0;
		let far = 2000.0;
		// let projection = Mat4::perspective(fov_y, aspect_ratio, near, far, (Hand::LH, Clip::NO));
		let focus_depth = position.distance(self.target_fast);
		let projection = Mat4::blend_ortho_perspective(self.blend, focus_depth, fov_y, aspect_ratio, near, far, (Hand::LH, Clip::NO));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
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
			inv_view_proj,
			clip: Clip::NO,
		}
	}
}

impl FxState {
	pub fn init_camera(&mut self) {
		self.camera.blend = 0.0;
		self.camera.offset = Vec3::new(0.0, 1.0 * 32.0, 200.0);

		self.camera.target = match self.camera.object.and_then(|h| self.objects.get(h)) {
			Some(obj) => obj.lerp_pos + Vec3(16.0, 16.0, 0.0),
			None => Vec3::ZERO,
		};
		self.camera.target_slow = self.camera.target;
		self.camera.target_fast = self.camera.target;
	}

	pub fn update_camera(&mut self, time: f32) {
		if self.camera.perspective {
			self.camera.blend = f32::clamp((time - 0.0) * 0.5, 0.0, 1.0);
		}
		else {
			self.camera.blend = 0.0;
		}

		if let Some(obj) = self.camera.object.and_then(|h| self.objects.get(h)) {
			self.camera.target = obj.lerp_pos + Vec3(16.0, 16.0 + 32.0 * self.camera.blend, 0.0)
		}

		self.camera.target_fast = self.camera.target_fast.exp_decay(self.camera.target, 25.0, 1.0 / 60.0);
		self.camera.target_slow = self.camera.target_slow.exp_decay(self.camera.target, 15.0, 1.0 / 60.0).set_x(self.camera.target_fast.x);
	}
}
