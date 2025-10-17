use super::*;

const FOV_Y: f32 = 90.0;
const NEAR: f32 = 10.0;
const FAR: f32 = 2000.0;

pub struct PlayCamera {
	// Camera offset from the object
	pub offset: Vec3f,

	// Camera look at target
	pub target: Vec3f,

	// Camera position
	// Slowly tracks target + offset
	pub position: Vec3f,

	// Blend between orthographic and perspective projection
	blend: f32,
	perspective: bool,
}

impl Default for PlayCamera {
	fn default() -> Self {
		Self {
			offset: Vec3::new(0.0, 1.0 * 32.0, 200.0),
			target: Vec3f::ZERO,
			position: Vec3f::ONE,
			blend: 0.0,
			perspective: false,
		}
	}
}

impl PlayCamera {
	pub fn setup(&self, screen_size: Vec2i) -> shade::d3::CameraSetup {
		let pos = self.target + self.get_offset();
		let corr = offset_correction(pos.y - self.target.y, pos.z, Angle::deg(FOV_Y));
		let corr = Vec3(0.0, corr, 0.0);
		let position = self.position + corr;
		let target = self.target + corr;
		let view = Transform3f::look_at(position, target, -Vec3f::Y, Hand::LH);
		let aspect_ratio = screen_size.x as f32 / screen_size.y as f32;
		let fov_y = Angle::deg(FOV_Y);
		let focus_depth = position.distance(target);
		let projection = Mat4::blend_ortho_perspective(self.blend, focus_depth, fov_y, aspect_ratio, NEAR, FAR, (Hand::LH, Clip::NO));
		let view_proj = projection * view;
		let inv_view_proj = view_proj.inverse();
		shade::d3::CameraSetup {
			surface: shade::Surface::BACK_BUFFER,
			viewport: Bounds2::vec(screen_size),
			aspect_ratio,
			position,
			view,
			near: NEAR,
			far: FAR,
			projection,
			view_proj,
			inv_view_proj,
			clip: Clip::NO,
		}
	}

	fn get_offset(&self) -> Vec3f {
		self.offset.set_y(self.offset.y * self.blend)
	}

	pub fn set_perspective(&mut self, perspective: bool) {
		self.perspective = perspective;
	}

	pub fn set_target(&mut self, pos: Vec3f) {
		self.target = pos;
	}

	pub fn teleport(&mut self, pos: Vec3f) {
		self.target = pos;
		self.position = pos + self.get_offset();
	}

	// Update blend over time
	pub fn animate_blend(&mut self) {
		if self.perspective {
			self.blend = f32::min(1.0, self.blend + 0.01);
		}
		else {
			self.blend = f32::max(0.0, self.blend - 0.01);
		}
	}

	pub fn animate_position(&mut self) {
		let position_target = self.target + self.get_offset();
		self.position = self.position.exp_decay(position_target, 15.0, 1.0 / 60.0).set_x(position_target.x);
	}
}

// When looking at the scene from an angle more space is visible above than below the target.
// This function computes an offset to apply to the camera and target position to keep the space above and below the target more balanced.
fn offset_correction(dx: f32, dy: f32, fov_y: Anglef) -> f32 {
	// Note: atan2 is defined as atan2(opposite, adjacent) which is why the arguments are swapped.
	let angle = Anglef::atan2(dx, dy);
	let angle_top = angle + fov_y * 0.5;
	let angle_bot = angle - fov_y * 0.5;

	let hit_top = dy * angle_top.tan();
	let hit_bot = dy * angle_bot.tan();

	((hit_top + hit_bot) * 0.5) - dx
}

#[test]
fn test_offset_correction() {
	let dy = 200.0;
	let fov_y = Angle::deg(90.0);

	let corr1 = offset_correction(-32.0, dy, fov_y);
	assert_eq!(corr1.round(), -34.0);
	let corr2 = offset_correction(0.0, dy, fov_y);
	assert_eq!(corr2, 0.0);
}
