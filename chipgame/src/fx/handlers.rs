use super::*;

fn ent_pos(gs: &chipcore::GameState, ent: &chipcore::Entity, pos: Vec2i) -> Vec3f {
	let terrain = gs.field.get_terrain(pos);
	let elevated = terrain.is_wall();
	// Blocks appear on top of walls
	let pos_z = if matches!(ent.kind, chipty::EntityKind::Block | chipty::EntityKind::IceBlock) { 0.0 } else if elevated { 20.0 } else { 0.0 };
	let pos = Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, pos_z);
	return pos;
}

pub fn entity_created(ctx: &mut FxState, ehandle: chipcore::EntityHandle, kind: chipty::EntityKind) {
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	let pos = ent_pos(&ctx.gs, ent, ent.pos);
	let obj = render::Object {
		data: render::ObjectData {
			pos,
			sprite: sprite_for_ent(ent, &ctx.gs.ps),
			model: if pos.z >= 20.0 { data::ModelId::FloorSprite } else { model_for_ent(ent) },
			alpha: 1.0,
			visible: true,
			greyscale: ent.flags & chipcore::EF_TEMPLATE != 0,
		},
		anim: render::Animation {
			anims: Vec::new(),
			unalive_after_anim: false,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
	ctx.objlookup.insert(ehandle, handle);

	if matches!(kind, chipty::EntityKind::Player) {
		ctx.camera.move_src = ent.pos;
		ctx.camera.move_dest = ent.pos;
		ctx.camera.move_time = ctx.render.time;
		ctx.camera.move_spd = ent.base_spd as f32 / 60.0;
		ctx.camera.move_teleport = true;
	}
}

pub fn entity_removed(ctx: &mut FxState, ehandle: chipcore::EntityHandle, kind: chipty::EntityKind) {
	let Some(obj_handle) = ctx.objlookup.remove(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	// Object rises, fades and is removed
	let rises = matches!(kind, chipty::EntityKind::Chip
		| chipty::EntityKind::BlueKey | chipty::EntityKind::RedKey | chipty::EntityKind::GreenKey | chipty::EntityKind::YellowKey
		| chipty::EntityKind::Flippers | chipty::EntityKind::FireBoots | chipty::EntityKind::IceSkates | chipty::EntityKind::SuctionBoots);

	// Object fades and is removed
	let faded = matches!(kind, chipty::EntityKind::Socket);

	if rises {
		obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
		obj.anim.anims.push(render::AnimState::MoveVel(render::MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) }));
	}
	else if faded {
		obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
	}
	obj.anim.unalive_after_anim = true;
}

pub fn entity_step(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	let src = ent.pos - match ent.step_dir { Some(step_dir) => step_dir.to_vec(), None => Vec2::ZERO };
	obj.data.pos = ent_pos(&ctx.gs, ent, src);
	obj.anim.anims.push(render::AnimState::MoveStep(render::MoveStep {
		dest: ent.pos,
		move_time: ctx.render.time,
		move_spd: ent.step_spd as f32 / 60.0,
	}));

	// Quick hack to flatten sprites on top of walls
	let check_pos = ent_pos(&ctx.gs, ent, ent.pos);
	obj.data.model = if check_pos.z >= 20.0 { data::ModelId::FloorSprite } else { model_for_ent(ent) };

	if ent.handle == ctx.gs.ps.ehandle {
		ctx.camera.move_src = src;
		ctx.camera.move_dest = ent.pos;
		ctx.camera.move_time = ctx.render.time;
		ctx.camera.move_spd = ent.step_spd as f32 / 60.0;
		ctx.camera.move_teleport = false;
	}
}

pub fn entity_teleport(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	// Step out of the teleport
	entity_step(ctx, ehandle);

	// When teleporting the player snap the camera
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };
	if ent.handle == ctx.gs.ps.ehandle {
		ctx.camera.move_teleport = true;
	}
}

pub fn entity_face_dir(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.data.sprite = sprite_for_ent(ent, &ctx.gs.ps);
}

pub fn player_activity(ctx: &mut FxState, _player: ()) {
	// Player cheer sprite
	let ehandle = ctx.gs.ps.ehandle;
	entity_face_dir(ctx, ehandle);

	// Play fireworks effect
	let Some(player) = ctx.gs.ents.get(ehandle) else { return };
	match ctx.gs.ps.activity {
		chipcore::PlayerActivity::Win => handlers::effect(ctx, player.pos, render::EffectType::Fireworks),
		_ => {}
	}
}

pub fn entity_hidden(ctx: &mut FxState, ehandle: chipcore::EntityHandle, hidden: bool) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	obj.data.visible = !hidden;
}

pub fn terrain_updated(ctx: &mut FxState, pos: Vec2i, old: chipty::Terrain, new: chipty::Terrain) {
	ctx.render.field.set_terrain(pos, new);
	match (old, new) {
		(chipty::Terrain::FakeBlueWall, _) => handlers::blue_wall_cleared(ctx, pos),
		(chipty::Terrain::ToggleFloor, chipty::Terrain::ToggleWall) => handlers::toggle_wall(ctx, pos),
		(chipty::Terrain::ToggleWall, chipty::Terrain::ToggleFloor) => handlers::toggle_wall(ctx, pos),
		(chipty::Terrain::ToggleFloor, _) => handlers::remove_toggle_wall(ctx, pos),
		(chipty::Terrain::ToggleWall, _) => handlers::remove_toggle_wall(ctx, pos),
		(_, chipty::Terrain::ToggleFloor) => handlers::create_toggle_wall(ctx, pos, false),
		(_, chipty::Terrain::ToggleWall) => handlers::create_toggle_wall(ctx, pos, true),
		(chipty::Terrain::Fire, _) => handlers::remove_fire(ctx, pos),
		(_, chipty::Terrain::Fire) => handlers::create_fire(ctx, pos),
		(chipty::Terrain::RecessedWall, chipty::Terrain::Wall) => handlers::recessed_wall_raised(ctx, pos),

		(chipty::Terrain::InvisibleWall, chipty::Terrain::HiddenWall) => {},
		(chipty::Terrain::HiddenWall, chipty::Terrain::InvisibleWall) => {},
		(chipty::Terrain::InvisibleWall | chipty::Terrain::HiddenWall, _) => handlers::remove_invis_wall(ctx, pos),
		(_, chipty::Terrain::InvisibleWall | chipty::Terrain::HiddenWall) => handlers::create_invis_wall(ctx, pos),
		_ => {}
	}
}

pub fn fire_hidden(ctx: &mut FxState, pos: Vec2i, hidden: bool) {
	let Some(&obj_handle) = ctx.firesprites.get(&pos) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	obj.data.visible = !hidden;
}

pub fn create_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0 - 2.0, 0.0), // Make fire appear below other sprites
			sprite: data::SpriteId::Fire,
			model: data::ModelId::Sprite,
			alpha: 1.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: Vec::new(),
			unalive_after_anim: false,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
	ctx.firesprites.insert(pos, handle);
}
pub fn remove_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	let Some(obj_handle) = ctx.firesprites.remove(&pos) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
	obj.anim.unalive_after_anim = true;
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
	let Some(obj_handle) = ctx.objlookup.remove(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
	obj.anim.anims.push(render::AnimState::MoveVel(render::MoveVel { vel: Vec3::new(0.0, 0.0, 200.0) }));
	obj.anim.unalive_after_anim = true;
}

pub fn lock_opened(ctx: &mut FxState, pos: Vec2<i32>, key: chipcore::KeyColor) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
			sprite: match key {
				chipcore::KeyColor::Red => data::SpriteId::RedLock,
				chipcore::KeyColor::Green => data::SpriteId::GreenLock,
				chipcore::KeyColor::Blue => data::SpriteId::BlueLock,
				chipcore::KeyColor::Yellow => data::SpriteId::YellowLock,
			},
			model: data::ModelId::Wall,
			alpha: 1.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: vec![render::AnimState::MoveZ(render::MoveZ {
				target_z: -21.0,
				move_spd: 200.0,
			})],
			unalive_after_anim: true,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
}

pub fn blue_wall_cleared(ctx: &mut FxState, pos: Vec2<i32>) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
			sprite: data::SpriteId::BlueWall,
			model: data::ModelId::Wall,
			alpha: 1.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: vec![render::AnimState::FadeOut(render::FadeOut { atime: 0.0 })],
			unalive_after_anim: true,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
}

pub fn recessed_wall_raised(ctx: &mut FxState, pos: Vec2<i32>) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
			sprite: data::SpriteId::Wall,
			model: data::ModelId::Wall,
			alpha: 1.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: vec![render::AnimState::MoveZ(render::MoveZ {
				target_z: 0.0,
				move_spd: 200.0,
			})],
			unalive_after_anim: false,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);

	// Keep the terrain as RecessedWall so that the wall object is drawn on top
	ctx.render.field.set_terrain(pos, chipty::Terrain::RecessedWall);
}

pub fn create_toggle_wall(ctx: &mut FxState, pos: Vec2<i32>, raised: bool) {
	let z = if raised { 0.0 } else { -21.0 };
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, z),
			sprite: data::SpriteId::Wall,
			model: data::ModelId::ThinWall,
			alpha: 1.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: vec![
				render::AnimState::MoveZ(render::MoveZ {
					target_z: if raised { 0.0 } else { -21.0 },
					move_spd: 200.0,
				}),
			],
			unalive_after_anim: false,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
	ctx.togglewalls.insert(pos, handle);
}

pub fn remove_toggle_wall(ctx: &mut FxState, pos: Vec2<i32>) {
	let Some(obj_handle) = ctx.togglewalls.remove(&pos) else { return };
	ctx.render.objects.remove(obj_handle);
}

pub fn toggle_wall(ctx: &mut FxState, pos: Vec2i) {
	let Some(&obj_handle) = ctx.togglewalls.get(&pos) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	let terrain = ctx.gs.field.get_terrain(pos);
	obj.anim.anims.clear();
	obj.anim.anims.push(render::AnimState::MoveZ(render::MoveZ {
		target_z: if matches!(terrain, chipty::Terrain::ToggleWall) { 0.0 } else { -21.0 },
		move_spd: 200.0,
	}));
}

pub fn create_invis_wall(ctx: &mut FxState, pos: Vec2<i32>) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
			sprite: data::SpriteId::Wall,
			model: data::ModelId::Wall,
			alpha: 0.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: Vec::new(),
			unalive_after_anim: false,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
	ctx.inviswalls.insert(pos, handle);
}

pub fn remove_invis_wall(ctx: &mut FxState, pos: Vec2<i32>) {
	let Some(obj_handle) = ctx.inviswalls.remove(&pos) else { return };

	ctx.render.objects.remove(obj_handle);
}

pub fn game_over(ctx: &mut FxState, _player: ()) {
	if matches!(ctx.gs.ps.activity, chipcore::PlayerActivity::Win) {
		ctx.game_realtime = ctx.render.time;
		ctx.next_level_load = ctx.render.time + 2.0;
		ctx.game_win = true;
	}
	else {
		ctx.game_realtime = ctx.render.time;
		ctx.next_level_load = ctx.render.time + 2.0;
		ctx.game_win = false;
	}
}

pub fn effect(ctx: &mut FxState, pos: Vec2i, ty: render::EffectType) {
	let pos = Vec3::new(pos.x as f32 * 32.0 + 16.0, pos.y as f32 * 32.0 + 16.0, 10.0);
	let start = ctx.render.time;
	ctx.render.effects.push(render::Effect { ty, pos, start });
}
