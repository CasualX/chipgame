use super::*;

pub fn entity_created(ctx: &mut VisualState, ehandle: core::EntityHandle, kind: core::EntityKind) {
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
		anim: Animation::None,
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

pub fn entity_removed(ctx: &mut VisualState, ehandle: core::EntityHandle, kind: core::EntityKind) {
	let Some(obj_handle) = ctx.objects.lookup.remove(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	// Object rises, fades and is removed
	let rises = matches!(kind, core::EntityKind::Chip
		| core::EntityKind::BlueKey | core::EntityKind::RedKey | core::EntityKind::GreenKey | core::EntityKind::YellowKey
		| core::EntityKind::Flippers | core::EntityKind::FireBoots | core::EntityKind::IceSkates | core::EntityKind::SuctionBoots);

	// Object fades and is removed
	let faded = matches!(kind, core::EntityKind::Socket);

	if rises {
		obj.anim = Animation::Rise;
		obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) });
		obj.unalive_after_anim = true;
	}
	else if faded {
		obj.anim = Animation::FadeOut;
		obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 0.0) });
		obj.unalive_after_anim = true;
	}
	else {
		// ctx.objects.remove(obj_handle);
		obj.unalive_after_anim = true;
	}
}

pub fn entity_step(ctx: &mut VisualState, ehandle: core::EntityHandle) {
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

pub fn entity_teleport(ctx: &mut VisualState, ehandle: core::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.pos = ent.pos.map(|c| c as f32 * 32.0).vec3(0.0);
	obj.lerp_pos = obj.pos;
	obj.mover = MoveType::Vel(MoveVel { vel: Vec3::ZERO });

	// ctx.events.push(Event::PlaySound(SoundFx::Teleporting));
}

pub fn entity_drown(ctx: &mut VisualState, ehandle: core::EntityHandle) {
	// let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	// let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	// ctx.events.push(Event::PlaySound(SoundFx::WaterSplash));
}

pub fn entity_face_dir(ctx: &mut VisualState, ehandle: core::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.sprite = sprite_for_ent(ent, &ctx.gs.ps);
}

pub fn player_activity(ctx: &mut VisualState, ehandle: core::EntityHandle) {
	entity_face_dir(ctx, ehandle);

	match ctx.gs.ps.activity {
		// core::PlayerActivity::Skating => ctx.events.push(Event::PlaySound(SoundFx::SkatingForward)),
		_ => (),
	}
}

pub fn entity_hidden(ctx: &mut VisualState, ehandle: core::EntityHandle, hidden: bool) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	obj.vis = !hidden;
}

pub fn create_fire(ctx: &mut VisualState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: Sprite::Fire,
		model: Model::Sprite,
		anim: Animation::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

pub fn create_toggle_floor(ctx: &mut VisualState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, -21.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, -21.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: Sprite::Wall,
		model: Model::ThinWall,
		anim: Animation::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

pub fn create_toggle_wall(ctx: &mut VisualState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: Sprite::Wall,
		model: Model::ThinWall,
		anim: Animation::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

fn model_for_ent(ent: &core::Entity) -> Model {
	match ent.kind {
		core::EntityKind::Block => Model::Wall,
		core::EntityKind::Tank => Model::ReallyFlatSprite,
		core::EntityKind::Bug => Model::FlatSprite,
		core::EntityKind::Blob => Model::ReallyFlatSprite,
		core::EntityKind::Paramecium => Model::ReallyFlatSprite,
		_ => Model::Sprite,
	}
}

fn sprite_for_ent(ent: &core::Entity, pl: &core::PlayerState) -> Sprite {
	match ent.kind {
		core::EntityKind::Player => match pl.activity {
			core::PlayerActivity::Walking | core::PlayerActivity::Pushing | core::PlayerActivity::Skating | core::PlayerActivity::Suction | core::PlayerActivity::Sliding =>
				match ent.face_dir {
					Some(core::Compass::Up) => Sprite::PlayerWalkUp,
					Some(core::Compass::Down) => Sprite::PlayerWalkDown,
					Some(core::Compass::Left) => Sprite::PlayerWalkLeft,
					Some(core::Compass::Right) => Sprite::PlayerWalkRight,
					_ => Sprite::PlayerWalkNeutral,
				},
			core::PlayerActivity::Win => Sprite::PlayerCheer,
			core::PlayerActivity::Swimming => match ent.face_dir {
				Some(core::Compass::Up) => Sprite::PlayerSwimUp,
				Some(core::Compass::Down) => Sprite::PlayerSwimDown,
				Some(core::Compass::Left) => Sprite::PlayerSwimLeft,
				Some(core::Compass::Right) => Sprite::PlayerSwimRight,
				_ => Sprite::PlayerSwimNeutral,
			},
			core::PlayerActivity::Drowned => Sprite::WaterSplash,
			core::PlayerActivity::Burned => Sprite::PlayerBurned,
			_ => Sprite::PlayerDead,
		},
		core::EntityKind::Chip => Sprite::Chip,
		core::EntityKind::Socket => Sprite::Socket,
		core::EntityKind::Block => Sprite::Block,
		core::EntityKind::Flippers => Sprite::PowerFlippers,
		core::EntityKind::FireBoots => Sprite::PowerFireBoots,
		core::EntityKind::IceSkates => Sprite::PowerIceSkates,
		core::EntityKind::SuctionBoots => Sprite::PowerSuctionBoots,
		core::EntityKind::BlueKey => Sprite::BlueKey,
		core::EntityKind::RedKey => Sprite::RedKey,
		core::EntityKind::GreenKey => Sprite::GreenKey,
		core::EntityKind::YellowKey => Sprite::YellowKey,
		core::EntityKind::Thief => Sprite::Thief,
		core::EntityKind::Bug => match ent.face_dir {
			Some(core::Compass::Up) => Sprite::BugUp,
			Some(core::Compass::Down) => Sprite::BugDown,
			Some(core::Compass::Left) => Sprite::BugLeft,
			Some(core::Compass::Right) => Sprite::BugRight,
			_ => Sprite::BugUp,
		},
		core::EntityKind::Tank => match ent.face_dir {
			Some(core::Compass::Up) => Sprite::TankUp,
			Some(core::Compass::Down) => Sprite::TankDown,
			Some(core::Compass::Left) => Sprite::TankLeft,
			Some(core::Compass::Right) => Sprite::TankRight,
			_ => Sprite::TankUp,
		},
		core::EntityKind::PinkBall => Sprite::PinkBall,
		core::EntityKind::FireBall => Sprite::FireBall,
		core::EntityKind::Glider => match ent.face_dir {
			Some(core::Compass::Up) => Sprite::GliderUp,
			Some(core::Compass::Down) => Sprite::GliderDown,
			Some(core::Compass::Left) => Sprite::GliderLeft,
			Some(core::Compass::Right) => Sprite::GliderRight,
			_ => Sprite::GliderUp,
		},
		core::EntityKind::Walker => match ent.face_dir {
			Some(core::Compass::Up) | Some(core::Compass::Down) => Sprite::WalkerUpDown,
			Some(core::Compass::Left) | Some(core::Compass::Right) => Sprite::WalkerLeftRight,
			_ => Sprite::WalkerUpDown,
		},
		core::EntityKind::Teeth => match ent.face_dir {
			Some(core::Compass::Up) => Sprite::TeethUp,
			Some(core::Compass::Down) => Sprite::TeethDown,
			Some(core::Compass::Left) => Sprite::TeethLeft,
			Some(core::Compass::Right) => Sprite::TeethRight,
			_ => Sprite::TeethUp,
		},
		core::EntityKind::Blob => Sprite::Blob,
		core::EntityKind::Paramecium => match ent.face_dir {
			Some(core::Compass::Up) | Some(core::Compass::Down) => Sprite::ParameciumUpDown,
			Some(core::Compass::Left) | Some(core::Compass::Right) => Sprite::ParameciumLeftRight,
			_ => Sprite::ParameciumUpDown,
		}
		core::EntityKind::Bomb => Sprite::Bomb,
	}
}

pub fn item_pickup(ctx: &mut VisualState, ehandle: core::EntityHandle, item: core::ItemPickup) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	obj.anim = Animation::Rise;
	obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) });

	// let sfx = match item {
	// 	core::ItemPickup::Chip => SoundFx::ICCollected,
	// 	core::ItemPickup::Flippers | core::ItemPickup::FireBoots | core::ItemPickup::IceSkates | core::ItemPickup::SuctionBoots => SoundFx::BootCollected,
	// 	core::ItemPickup::BlueKey | core::ItemPickup::RedKey | core::ItemPickup::GreenKey | core::ItemPickup::YellowKey => SoundFx::KeyCollected,
	// };
	// ctx.events.push(Event::PlaySound(sfx));
}

pub fn lock_opened(ctx: &mut VisualState, pos: Vec2<i32>, key: core::KeyColor) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, -200.0) }),
		sprite: match key {
			core::KeyColor::Red => Sprite::RedLock,
			core::KeyColor::Green => Sprite::GreenLock,
			core::KeyColor::Blue => Sprite::BlueLock,
			core::KeyColor::Yellow => Sprite::YellowLock,
		},
		model: Model::Wall,
		anim: Animation::Fall,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: true,
	};
	ctx.objects.insert(obj);
	// ctx.events.push(Event::PlaySound(SoundFx::LockOpened));
}

pub fn blue_wall_cleared(ctx: &mut VisualState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: Sprite::BlueWall,
		model: Model::Wall,
		anim: Animation::FadeOut,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: true,
	};
	ctx.objects.insert(obj);
	// ctx.events.push(Event::PlaySound(SoundFx::BlueWallCleared));
}

pub fn hidden_wall_bumped(ctx: &mut VisualState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: Sprite::Wall,
		model: Model::Wall,
		anim: Animation::FadeIn,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
}

pub fn recessed_wall_raised(ctx: &mut VisualState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		ehandle: core::EntityHandle::INVALID,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 0.0) }),
		sprite: Sprite::Wall,
		model: Model::Wall,
		anim: Animation::Raise,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
	};
	ctx.objects.insert(obj);
	// ctx.events.push(Event::PlaySound(SoundFx::WallPopup));
}

pub fn toggle_walls(ctx: &mut VisualState) {
	for obj in ctx.objects.map.values_mut() {
		if obj.model != Model::ThinWall {
			continue;
		}

		let pos = obj.pos.xy().map(|c| (c / 32.0) as i32);
		let terrain = ctx.gs.field.get_terrain(pos);
		if matches!(terrain, core::Terrain::ToggleFloor) {
			obj.pos.z = 0.0;
			obj.anim = Animation::Fall;
			obj.mover = MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, -200.0) });
		}
		else if matches!(terrain, core::Terrain::ToggleWall) {
			obj.pos.z = -21.0;
			obj.anim = Animation::Raise;
			obj.mover = MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, 200.0) });
		}
	}
}

pub fn button_press(ctx: &mut VisualState) {
	// ctx.events.push(Event::PlaySound(SoundFx::ButtonPressed));
}

pub fn game_win(ctx: &mut VisualState) {
	// ctx.events.push(Event::PlaySound(SoundFx::GameWin));

	ctx.next_level_load = ctx.time + 2.0;
}

pub fn socket_filled(ctx: &mut VisualState, _pos: Vec2<i32>) {
	// ctx.events.push(Event::PlaySound(SoundFx::SocketOpened));
}

pub fn player_bump(ctx: &mut VisualState, _player: core::EntityHandle) {
	// ctx.events.push(Event::PlaySound(SoundFx::CantMove));
}

pub fn block_push(ctx: &mut VisualState, _entity: core::EntityHandle) {
	// ctx.events.push(Event::PlaySound(SoundFx::BlockMoving));
}

pub fn entity_trapped(ctx: &mut VisualState, _entity: core::EntityHandle) {
	// ctx.events.push(Event::PlaySound(SoundFx::TrapEntered));
}

pub fn bomb_explode(ctx: &mut VisualState, _entity: core::EntityHandle) {
	// ctx.events.push(Event::PlaySound(SoundFx::BombExplodes));
}

pub fn items_thief(ctx: &mut VisualState, _player: core::EntityHandle) {
	// ctx.events.push(Event::PlaySound(SoundFx::BootsStolen));
}

pub fn dirt_cleared(ctx: &mut VisualState, _pos: Vec2i) {
	// ctx.events.push(Event::PlaySound(SoundFx::TileEmptied));
}
