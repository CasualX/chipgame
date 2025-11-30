use super::*;

const FOV_Y: f32 = 90.0;
const NEAR: f32 = 10.0;
const FAR: f32 = 2000.0;

#[derive(Clone)]
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

	pub move_src: Vec2<i32>,
	pub move_dest: Vec2<i32>,
	pub move_time: f64,
	pub move_spd: f32,
	pub move_teleport: bool,
}

impl Default for PlayCamera {
	fn default() -> Self {
		Self {
			offset: Vec3::new(0.0, 1.0 * 32.0, 200.0),
			target: Vec3f::ZERO,
			position: Vec3f::ONE,
			blend: 0.0,
			perspective: false,
			move_src: Vec2::ZERO,
			move_dest: Vec2::ZERO,
			move_time: 0.0,
			move_spd: 1.0,
			move_teleport: true,
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

	// Update blend over time
	pub fn animate_blend(&mut self) {
		if self.perspective {
			self.blend = f32::min(1.0, self.blend + 0.01);
		}
		else {
			self.blend = f32::max(0.0, self.blend - 0.01);
		}
	}

	pub fn animate_position(&mut self, dt: f64) {
		let position_target = self.target + self.get_offset();
		self.position = self.position.exp_decay(position_target, 15.0, dt as f32).set_x(position_target.x);
	}

	pub fn animate_move(&mut self, time: f64) {
		let t = f32::min(1.0, (time - self.move_time) as f32 / self.move_spd);
		let src = self.move_src.map(|c| c as f32 * 32.0 + 16.0).vec3(0.0);
		let dest = self.move_dest.map(|c| c as f32 * 32.0 + 16.0).vec3(0.0);
		let new_target = src.lerp(dest, t);
		self.target = new_target;
		if self.move_teleport {
			self.position = new_target + self.get_offset();
			self.move_teleport = false;
		}
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
