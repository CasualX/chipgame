use super::*;

fn ent_pos(gs: &chipcore::GameState, ent: &chipcore::Entity, pos: Vec2i) -> Vec3f {
	let terrain = gs.field.get_terrain(pos);
	let elevated = terrain.is_wall();
	// Blocks appear on top of walls
	let pos_z = if matches!(ent.kind, chipty::EntityKind::Block | chipty::EntityKind::IceBlock) { 1.0 } else if elevated { 20.0 } else { 0.0 };
	let pos = Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, pos_z);
	return pos;
}

pub fn entity_created(ctx: &mut FxState, ehandle: chipcore::EntityHandle, kind: chipty::EntityKind) {
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };
	let handle = ctx.objects.alloc();
	let pos = ent_pos(&ctx.gs, ent, ent.pos);
	// Quick hack to flatten sprites on top of walls
	let model = if pos.z >= 20.0 { data::ModelId::FloorSprite } else { model_for_ent(ent) };
	let greyscale = ent.flags & chipcore::EF_TEMPLATE != 0;
	let obj = Object {
		handle,
		pos,
		lerp_pos: pos,
		mover: MoveType::Vel(MoveVel { vel: Vec3::ZERO }),
		sprite: sprite_for_ent(ent, &ctx.gs.ps),
		model,
		anim: data::AnimationId::None,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: false,
		greyscale,
	};
	ctx.objects.insert(obj);
	ctx.objects.lookup.insert(ehandle, handle);

	if matches!(kind, chipty::EntityKind::Player) {
		ctx.camera.teleport(pos + Vec3(16.0, 16.0, 0.0));
	}
}

pub fn entity_removed(ctx: &mut FxState, ehandle: chipcore::EntityHandle, kind: chipty::EntityKind) {
	let Some(obj_handle) = ctx.objects.lookup.remove(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	// Object rises, fades and is removed
	let rises = matches!(kind, chipty::EntityKind::Chip
		| chipty::EntityKind::BlueKey | chipty::EntityKind::RedKey | chipty::EntityKind::GreenKey | chipty::EntityKind::YellowKey
		| chipty::EntityKind::Flippers | chipty::EntityKind::FireBoots | chipty::EntityKind::IceSkates | chipty::EntityKind::SuctionBoots);

	// Object fades and is removed
	let faded = matches!(kind, chipty::EntityKind::Socket);

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

pub fn entity_step(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	let src = ent.pos - match ent.step_dir { Some(step_dir) => step_dir.to_vec(), None => Vec2::ZERO };
	obj.pos = ent_pos(&ctx.gs, ent, src);
	obj.mover = MoveType::Step(MoveStep {
		src,
		dest: ent.pos,
		move_time: ctx.time,
		move_spd: ent.step_spd as f32 / 60.0,
	});
	// Quick hack to flatten sprites on top of walls
	obj.model = if obj.pos.z >= 20.0 { data::ModelId::FloorSprite } else { model_for_ent(ent) };
}

pub fn entity_teleport(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	// Step out of the teleport
	entity_step(ctx, ehandle);

	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	// When teleporting the player snap the camera
	if ent.handle == ctx.gs.ps.ehandle {
		obj.lerp_pos = obj.pos;
		ctx.camera.teleport(obj.lerp_pos + Vec3(16.0, 16.0, 0.0));
	}
}

pub fn entity_drown(_ctx: &mut FxState, _ehandle: chipcore::EntityHandle) {
	// let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	// let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
}

pub fn entity_face_dir(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.sprite = sprite_for_ent(ent, &ctx.gs.ps);
}

pub fn player_activity(ctx: &mut FxState, _player: ()) {
	let ehandle = ctx.gs.ps.ehandle;
	entity_face_dir(ctx, ehandle);
}

pub fn entity_hidden(ctx: &mut FxState, ehandle: chipcore::EntityHandle, hidden: bool) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	obj.vis = !hidden;
}

pub fn fire_hidden(ctx: &mut FxState, pos: Vec2i, hidden: bool) {
	let Some(obj) = ctx.objects.map.values_mut().find(|obj| {
		obj.sprite == data::SpriteId::Fire && (obj.pos.xy() + Vec2(0.0, 2.0)).map(|c| (c / 32.0).round() as i32) == pos
	}) else { return };
	obj.vis = !hidden;
}

pub fn create_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
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
		greyscale: false,
	};
	ctx.objects.insert(obj);
}
pub fn remove_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	for obj in ctx.objects.map.values_mut() {
		// Fire sprites are drawn 2px lower (y-2) to appear under other sprites
		// This is a horrible hack! Figure out a better way to find fire objects related to a position
		if obj.sprite == data::SpriteId::Fire && (obj.pos.xy() + Vec2(0.0, 2.0)).map(|c| (c / 32.0).round() as i32) == pos {
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
		greyscale: false,
	};
	ctx.objects.insert(obj);
}

pub fn create_toggle_wall(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
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
		greyscale: false,
	};
	ctx.objects.insert(obj);
}

fn model_for_ent(ent: &chipcore::Entity) -> data::ModelId {
	match ent.kind {
		chipty::EntityKind::Block => data::ModelId::Wall,
		chipty::EntityKind::IceBlock => data::ModelId::Wall,
		chipty::EntityKind::Tank => data::ModelId::ReallyFlatSprite,
		chipty::EntityKind::Bug => data::ModelId::FlatSprite,
		chipty::EntityKind::Blob => data::ModelId::ReallyFlatSprite,
		chipty::EntityKind::Paramecium => data::ModelId::ReallyFlatSprite,
		_ => data::ModelId::Sprite,
	}
}

fn sprite_for_ent(ent: &chipcore::Entity, pl: &chipcore::PlayerState) -> data::SpriteId {
	match ent.kind {
		chipty::EntityKind::Player => match pl.activity {
			chipcore::PlayerActivity::Walking | chipcore::PlayerActivity::Pushing | chipcore::PlayerActivity::Skating | chipcore::PlayerActivity::Suction | chipcore::PlayerActivity::Sliding =>
				match ent.face_dir {
					Some(chipty::Compass::Up) => data::SpriteId::PlayerWalkUp,
					Some(chipty::Compass::Down) => data::SpriteId::PlayerWalkDown,
					Some(chipty::Compass::Left) => data::SpriteId::PlayerWalkLeft,
					Some(chipty::Compass::Right) => data::SpriteId::PlayerWalkRight,
					_ => data::SpriteId::PlayerWalkNeutral,
				},
			chipcore::PlayerActivity::Win => data::SpriteId::PlayerCheer,
			chipcore::PlayerActivity::Swimming => match ent.face_dir {
				Some(chipty::Compass::Up) => data::SpriteId::PlayerSwimUp,
				Some(chipty::Compass::Down) => data::SpriteId::PlayerSwimDown,
				Some(chipty::Compass::Left) => data::SpriteId::PlayerSwimLeft,
				Some(chipty::Compass::Right) => data::SpriteId::PlayerSwimRight,
				_ => data::SpriteId::PlayerSwimNeutral,
			},
			chipcore::PlayerActivity::Drowned => data::SpriteId::WaterSplash,
			chipcore::PlayerActivity::Burned => data::SpriteId::PlayerBurned,
			_ => data::SpriteId::PlayerDead,
		},
		chipty::EntityKind::Chip => data::SpriteId::Chip,
		chipty::EntityKind::Socket => data::SpriteId::Socket,
		chipty::EntityKind::Block => data::SpriteId::DirtBlock,
		chipty::EntityKind::IceBlock => data::SpriteId::IceBlock,
		chipty::EntityKind::Flippers => data::SpriteId::Flippers,
		chipty::EntityKind::FireBoots => data::SpriteId::FireBoots,
		chipty::EntityKind::IceSkates => data::SpriteId::IceSkates,
		chipty::EntityKind::SuctionBoots => data::SpriteId::SuctionBoots,
		chipty::EntityKind::BlueKey => data::SpriteId::BlueKey,
		chipty::EntityKind::RedKey => data::SpriteId::RedKey,
		chipty::EntityKind::GreenKey => data::SpriteId::GreenKey,
		chipty::EntityKind::YellowKey => data::SpriteId::YellowKey,
		chipty::EntityKind::Thief => data::SpriteId::Thief,
		chipty::EntityKind::Bug => match ent.face_dir {
			Some(chipty::Compass::Up) => data::SpriteId::BugUp,
			Some(chipty::Compass::Down) => data::SpriteId::BugDown,
			Some(chipty::Compass::Left) => data::SpriteId::BugLeft,
			Some(chipty::Compass::Right) => data::SpriteId::BugRight,
			_ => data::SpriteId::BugUp,
		},
		chipty::EntityKind::Tank => match ent.face_dir {
			Some(chipty::Compass::Up) => data::SpriteId::TankUp,
			Some(chipty::Compass::Down) => data::SpriteId::TankDown,
			Some(chipty::Compass::Left) => data::SpriteId::TankLeft,
			Some(chipty::Compass::Right) => data::SpriteId::TankRight,
			_ => data::SpriteId::TankUp,
		},
		chipty::EntityKind::PinkBall => data::SpriteId::PinkBall,
		chipty::EntityKind::FireBall => data::SpriteId::FireBall,
		chipty::EntityKind::Glider => match ent.face_dir {
			Some(chipty::Compass::Up) => data::SpriteId::GliderUp,
			Some(chipty::Compass::Down) => data::SpriteId::GliderDown,
			Some(chipty::Compass::Left) => data::SpriteId::GliderLeft,
			Some(chipty::Compass::Right) => data::SpriteId::GliderRight,
			_ => data::SpriteId::GliderUp,
		},
		chipty::EntityKind::Walker => match ent.face_dir {
			Some(chipty::Compass::Up) | Some(chipty::Compass::Down) => data::SpriteId::WalkerUpDown,
			Some(chipty::Compass::Left) | Some(chipty::Compass::Right) => data::SpriteId::WalkerLeftRight,
			_ => data::SpriteId::WalkerUpDown,
		},
		chipty::EntityKind::Teeth => match ent.face_dir {
			Some(chipty::Compass::Up) => data::SpriteId::TeethUp,
			Some(chipty::Compass::Down) => data::SpriteId::TeethDown,
			Some(chipty::Compass::Left) => data::SpriteId::TeethLeft,
			Some(chipty::Compass::Right) => data::SpriteId::TeethRight,
			_ => data::SpriteId::TeethUp,
		},
		chipty::EntityKind::Blob => data::SpriteId::Blob,
		chipty::EntityKind::Paramecium => match ent.face_dir {
			Some(chipty::Compass::Up) | Some(chipty::Compass::Down) => data::SpriteId::ParameciumUpDown,
			Some(chipty::Compass::Left) | Some(chipty::Compass::Right) => data::SpriteId::ParameciumLeftRight,
			_ => data::SpriteId::ParameciumUpDown,
		}
		chipty::EntityKind::Bomb => data::SpriteId::Bomb,
	}
}

pub fn item_pickup(ctx: &mut FxState, ehandle: chipcore::EntityHandle, _item: chipcore::ItemPickup) {
	let Some(&obj_handle) = ctx.objects.lookup.get(&ehandle) else { return };
	let Some(obj) = ctx.objects.get_mut(obj_handle) else { return };

	obj.anim = data::AnimationId::Rise;
	obj.mover = MoveType::Vel(MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) });
}

pub fn lock_opened(ctx: &mut FxState, pos: Vec2<i32>, key: chipcore::KeyColor) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
		pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		lerp_pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
		mover: MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, -200.0) }),
		sprite: match key {
			chipcore::KeyColor::Red => data::SpriteId::RedLock,
			chipcore::KeyColor::Green => data::SpriteId::GreenLock,
			chipcore::KeyColor::Blue => data::SpriteId::BlueLock,
			chipcore::KeyColor::Yellow => data::SpriteId::YellowLock,
		},
		model: data::ModelId::Wall,
		anim: data::AnimationId::Fall,
		atime: 0.0,
		alpha: 1.0,
		vis: true,
		live: true,
		unalive_after_anim: true,
		greyscale: false,
	};
	ctx.objects.insert(obj);
}

pub fn blue_wall_cleared(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
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
		greyscale: false,
	};
	ctx.objects.insert(obj);
}

pub fn hidden_wall_bumped(ctx: &mut FxState, pos: Vec2<i32>) {
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
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
		greyscale: false,
	};
	ctx.objects.insert(obj);
}

pub fn recessed_wall_raised(ctx: &mut FxState, pos: Vec2<i32>) {
	// The raising animation won't be visible because a wall tile is drawn on top of it...
	let handle = ctx.objects.alloc();
	let obj = Object {
		handle,
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
		greyscale: false,
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
		if matches!(terrain, chipty::Terrain::ToggleFloor) {
			obj.pos.z = 0.0;
			obj.anim = data::AnimationId::Fall;
			obj.mover = MoveType::Vel(MoveVel { vel: Vec3(0.0, 0.0, -200.0) });
		}
		else if matches!(terrain, chipty::Terrain::ToggleWall) {
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

pub fn effect(ctx: &mut FxState, pos: Vec2i, ty: EffectType) {
	ctx.effects.push(Effect {
		ty,
		pos: Vec3::new(pos.x as f32 * 32.0 + 16.0, pos.y as f32 * 32.0 + 16.0, 10.0),
		start: ctx.time,
	});
}
