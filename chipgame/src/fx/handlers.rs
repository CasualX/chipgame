use super::*;

pub fn entity_created(ctx: &mut FxState, ehandle: core::EntityHandle, kind: core::EntityKind) {
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: ent.handle,
		pos: Vec3::new(ent.pos.x as f32 * 32.0, ent.pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(ent.pos.x as f32 * 32.0, ent.pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3::ZERO }),
		sprite: sprite_for_ent(ent, &ctx.gs.ps),
		model: model_for_ent(ent),
		anim: data::AnimationId::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	if matches!(kind, core::EntityKind::Player) {
		ctx.camera.object_h = Some(handle);
		ctx.camera.target = obj.pos;
		ctx.camera.target_fast = obj.pos;
	}
	ctx.objects.insert(obj);
	ctx.objects.lookup.insert(ent.handle, handle);
}

pub fn entity_removed(ctx: &mut FxState, ehandle: core::EntityHandle, kind: core::EntityKind) {
	let Some(obj_handle) = ctx.objects.lookup.remove(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	// Object rises, fades and is removed
	let rises = matches!(kind, core::EntityKind::Chip
		| core::EntityKind::BlueKey | core::EntityKind::RedKey | core::EntityKind::GreenKey | core::EntityKind::YellowKey
		| core::EntityKind::Flippers | core::EntityKind::FireBoots | core::EntityKind::IceSkates | core::EntityKind::SuctionBoots);

	// Object fades and is removed
	let faded = matches!(kind, core::EntityKind::Socket);

	if rises {
		obj.anim = data::AnimationId::Rise;
		obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) });
		obj.unalive_after_anim = true;
	}
	else if faded {
		obj.anim = data::AnimationId::FadeOut;
		obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 0.0) });
		obj.unalive_after_anim = true;
	}
	else {
		// ctx.objects.remove(obj_handle);
		obj.unalive_after_anim = true;
	}
}

pub fn entity_step(ctx: &mut FxState, ehandle: core::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	let src = ent.pos - match ent.step_dir { Some(step_dir) => step_dir.to_vec(), None => Vec2::ZERO };
	obj.pos = src.map(|c| c as f32 * 32.0).vec3(0.0);
	obj.mover = MoveType::Step(MoveStep {
		src,
		dest: ent.pos,
		move_time: ctx.time,
		move_spd: ent.step_spd as f32 / 60.0,
	});
}

pub fn entity_teleport(ctx: &mut FxState, ehandle: core::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.pos = ent.pos.map(|c| c as f32 * 32.0).vec3(0.0);
	obj.lerp_pos = obj.pos;
	obj.mover = MoveType::Vel(MoveVel { vel: Vec3::ZERO });
}

pub fn entity_drown(_ctx: &mut FxState, _ehandle: core::EntityHandle) {
	// let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	// let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
}

pub fn entity_face_dir(ctx: &mut FxState, ehandle: core::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.sprite = sprite_for_ent(ent, &ctx.gs.ps);
}

pub fn player_activity(ctx: &mut FxState, _player: ()) {
	let ehandle = ctx.gs.ps.ehandle;
	entity_face_dir(ctx, ehandle);
}

pub fn entity_hidden(ctx: &mut FxState, ehandle: core::EntityHandle, hidden: bool) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	obj.vis = !hidden;
}

pub fn create_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0 - 2.0, 0.0), // Make fire appear below other sprites
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: data::SpriteId::Fire,
		model: data::ModelId::Sprite,
		anim: data::AnimationId::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}
pub fn remove_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	for obj in ctx.objects.map.values_mut() {
		if obj.pos.xy().map(|c| (c / 32.0) as i32) == pos {
			obj.anim = data::AnimationId::FadeOut;
			obj.mover = MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) });
			obj.unalive_after_anim = true;
		}
	}
}

pub fn create_toggle_floor(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, -21.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, -21.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: data::SpriteId::Wall,
		model: data::ModelId::ThinWall,
		anim: data::AnimationId::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

pub fn create_toggle_wall(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: data::SpriteId::Wall,
		model: data::ModelId::ThinWall,
		anim: data::AnimationId::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

fn model_for_ent(ent: &core::Entity) -> data::ModelId {
	match ent.kind {
		core::EntityKind::Block => data::ModelId::Wall,
		core::EntityKind::Tank => data::ModelId::ReallyFlatSprite,
		core::EntityKind::Bug => data::ModelId::FlatSprite,
		core::EntityKind::Blob => data::ModelId::ReallyFlatSprite,
		core::EntityKind::Paramecium => data::ModelId::ReallyFlatSprite,
		_ => data::ModelId::Sprite,
	}
}

fn sprite_for_ent(ent: &core::Entity, pl: &core::PlayerState) -> data::SpriteId {
	match ent.kind {
		core::EntityKind::Player => match pl.activity {
			core::PlayerActivity::Walking | core::PlayerActivity::Pushing | core::PlayerActivity::Skating | core::PlayerActivity::Suction | core::PlayerActivity::Sliding =>
				match ent.face_dir {
					Some(core::Compass::Up) => data::SpriteId::PlayerWalkUp,
					Some(core::Compass::Down) => data::SpriteId::PlayerWalkDown,
					Some(core::Compass::Left) => data::SpriteId::PlayerWalkLeft,
					Some(core::Compass::Right) => data::SpriteId::PlayerWalkRight,
					_ => data::SpriteId::PlayerWalkNeutral,
				},
			core::PlayerActivity::Win => data::SpriteId::PlayerCheer,
			core::PlayerActivity::Swimming => match ent.face_dir {
				Some(core::Compass::Up) => data::SpriteId::PlayerSwimUp,
				Some(core::Compass::Down) => data::SpriteId::PlayerSwimDown,
				Some(core::Compass::Left) => data::SpriteId::PlayerSwimLeft,
				Some(core::Compass::Right) => data::SpriteId::PlayerSwimRight,
				_ => data::SpriteId::PlayerSwimNeutral,
			},
			core::PlayerActivity::Drowned => data::SpriteId::WaterSplash,
			core::PlayerActivity::Burned => data::SpriteId::PlayerBurned,
			_ => data::SpriteId::PlayerDead,
		},
		core::EntityKind::Chip => data::SpriteId::Chip,
		core::EntityKind::Socket => data::SpriteId::Socket,
		core::EntityKind::Block => data::SpriteId::DirtBlock,
		core::EntityKind::Flippers => data::SpriteId::Flippers,
		core::EntityKind::FireBoots => data::SpriteId::FireBoots,
		core::EntityKind::IceSkates => data::SpriteId::IceSkates,
		core::EntityKind::SuctionBoots => data::SpriteId::SuctionBoots,
		core::EntityKind::BlueKey => data::SpriteId::BlueKey,
		core::EntityKind::RedKey => data::SpriteId::RedKey,
		core::EntityKind::GreenKey => data::SpriteId::GreenKey,
		core::EntityKind::YellowKey => data::SpriteId::YellowKey,
		core::EntityKind::Thief => data::SpriteId::Thief,
		core::EntityKind::Bug => match ent.face_dir {
			Some(core::Compass::Up) => data::SpriteId::BugUp,
			Some(core::Compass::Down) => data::SpriteId::BugDown,
			Some(core::Compass::Left) => data::SpriteId::BugLeft,
			Some(core::Compass::Right) => data::SpriteId::BugRight,
			_ => data::SpriteId::BugUp,
		},
		core::EntityKind::Tank => match ent.face_dir {
			Some(core::Compass::Up) => data::SpriteId::TankUp,
			Some(core::Compass::Down) => data::SpriteId::TankDown,
			Some(core::Compass::Left) => data::SpriteId::TankLeft,
			Some(core::Compass::Right) => data::SpriteId::TankRight,
			_ => data::SpriteId::TankUp,
		},
		core::EntityKind::PinkBall => data::SpriteId::PinkBall,
		core::EntityKind::FireBall => data::SpriteId::FireBall,
		core::EntityKind::Glider => match ent.face_dir {
			Some(core::Compass::Up) => data::SpriteId::GliderUp,
			Some(core::Compass::Down) => data::SpriteId::GliderDown,
			Some(core::Compass::Left) => data::SpriteId::GliderLeft,
			Some(core::Compass::Right) => data::SpriteId::GliderRight,
			_ => data::SpriteId::GliderUp,
		},
		core::EntityKind::Walker => match ent.face_dir {
			Some(core::Compass::Up) | Some(core::Compass::Down) => data::SpriteId::WalkerUpDown,
			Some(core::Compass::Left) | Some(core::Compass::Right) => data::SpriteId::WalkerLeftRight,
			_ => data::SpriteId::WalkerUpDown,
		},
		core::EntityKind::Teeth => match ent.face_dir {
			Some(core::Compass::Up) => data::SpriteId::TeethUp,
			Some(core::Compass::Down) => data::SpriteId::TeethDown,
			Some(core::Compass::Left) => data::SpriteId::TeethLeft,
			Some(core::Compass::Right) => data::SpriteId::TeethRight,
			_ => data::SpriteId::TeethUp,
		},
		core::EntityKind::Blob => data::SpriteId::Blob,
		core::EntityKind::Paramecium => match ent.face_dir {
			Some(core::Compass::Up) | Some(core::Compass::Down) => data::SpriteId::ParameciumUpDown,
			Some(core::Compass::Left) | Some(core::Compass::Right) => data::SpriteId::ParameciumLeftRight,
			_ => data::SpriteId::ParameciumUpDown,
		}
		core::EntityKind::Bomb => data::SpriteId::Bomb,
	}
}

pub fn item_pickup(ctx: &mut FxState, ehandle: core::EntityHandle, _item: core::ItemPickup) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	obj.anim = data::AnimationId::Rise;
	obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) });
}

pub fn lock_opened(ctx: &mut FxState, pos: Vec2<i32>, key: core::KeyColor) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, -200.0) }),
		sprite: match key {
			core::KeyColor::Red => data::SpriteId::RedLock,
			core::KeyColor::Green => data::SpriteId::GreenLock,
			core::KeyColor::Blue => data::SpriteId::BlueLock,
			core::KeyColor::Yellow => data::SpriteId::YellowLock,
		},
		model: data::ModelId::Wall,
		anim: data::AnimationId::Fall,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: true,
	};
	ctx.objects.insert(obj);
}

pub fn blue_wall_cleared(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: data::SpriteId::BlueWall,
		model: data::ModelId::Wall,
		anim: data::AnimationId::FadeOut,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: true,
	};
	ctx.objects.insert(obj);
}

pub fn hidden_wall_bumped(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: data::SpriteId::Wall,
		model: data::ModelId::Wall,
		anim: data::AnimationId::FadeIn,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

pub fn recessed_wall_raised(ctx: &mut FxState, pos: Vec2<i32>) {
	// The raising animation won't be visible because a wall tile is drawn on top of it...
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: data::SpriteId::Wall,
		model: data::ModelId::Wall,
		anim: data::AnimationId::Raise,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

pub fn toggle_walls(ctx: &mut FxState) {
	for obj in ctx.objects.map.values_mut() {
		if obj.model != data::ModelId::ThinWall {
			continue;
		}

		let pos = obj.pos.xy().map(|c| (c / 32.0) as i32);
		let terrain = ctx.gs.field.get_terrain(pos);
		if matches!(terrain, core::Terrain::ToggleFloor) {
			obj.pos.z = 0.0;
			obj.anim = data::AnimationId::Fall;
			obj.mover = MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, -200.0) });
		}
		else if matches!(terrain, core::Terrain::ToggleWall) {
			obj.pos.z = -21.0;
			obj.anim = data::AnimationId::Raise;
			obj.mover = MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 200.0) });
		}
	}
}

pub fn game_win(ctx: &mut FxState) {
	ctx.gs_realtime = ctx.time;
	ctx.next_level_load = ctx.time + 2.0;
	ctx.game_win = true;
}

pub fn game_over(ctx: &mut FxState) {
	ctx.gs_realtime = ctx.time;
	ctx.next_level_load = ctx.time + 2.0;
	ctx.game_win = false;
}
