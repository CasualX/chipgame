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

	// Camera matrices
	pub view_mat: Mat4<f32>,
	pub proj_mat: Mat4<f32>,
	pub view_proj_mat: Mat4<f32>,
}

impl FxState {
	pub fn set_game_camera(&mut self, resx: &Resources) {
		let size = resx.screen_size;

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

		self.camera.proj_mat = Mat4::perspective_fov(Deg(45.0), size.x as f32, size.y as f32, 0.1, 2000.0, (RH, NO));
		self.camera.view_mat = {
			let eye = self.camera.target + self.camera.eye_offset;
			let target = self.camera.target_fast;
			let up = Vec3(0.0, -1.0, 0.0);
			Mat4::look_at(eye, target, up, RH)
		};

		self.camera.view_proj_mat = self.camera.proj_mat * self.camera.view_mat;
	}
}
