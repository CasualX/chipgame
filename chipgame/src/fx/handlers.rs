use super::*;

fn ent_pos(gs: &chipcore::GameState, ent: &chipcore::Entity, pos: Vec2i, check_elevated: bool) -> Vec3f {
	let terrain = gs.field.get_terrain(pos);
	let elevated = terrain.is_wall() && !matches!(terrain, chipty::Terrain::DirtBlock);
	// Blocks appear on top of walls
	let pos_z = if matches!(ent.kind, chipty::EntityKind::Block | chipty::EntityKind::IceBlock) { 0.0 } else if check_elevated && elevated { 20.0 } else { 0.0 };
	let pos = Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, pos_z);
	return pos;
}

pub fn entity_created(ctx: &mut FxState, ehandle: chipcore::EntityHandle, kind: chipty::EntityKind) {
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	let pos = ent_pos(&ctx.gs, ent, ent.pos, true);
	let sprite = sprite_for_ent(ent, &ctx.gs);
	let mut obj = render::Object {
		data: render::ObjectData {
			pos,
			sprite,
			frame: 0,
			model: if pos.z >= 20.0 { chipty::ModelId::FloorSprite } else { model_for_ent(ent) },
			alpha: 1.0,
			visible: true,
			greyscale: ent.flags & chipcore::EF_TEMPLATE != 0,
		},
		anim: render::Animation {
			anims: Vec::new(),
			unalive_after_anim: false,
		},
	};
	if matches!(kind, chipty::EntityKind::Bomb) {
		obj.data.sprite = chipty::SpriteId::BombA;
		obj.anim.anims.push(render::AnimState::AnimLoop(render::SpriteAnimLoop {
			start_time: ctx.time + ctx.random.next_f64() * 10.0,
			frame_rate: 16.0,
		}));
	}

	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
	ctx.objlookup.insert(ehandle, handle);

	if ctx.gs.ps.master == ehandle {
		ctx.camera.move_src = ent.pos;
		ctx.camera.move_dest = ent.pos;
		ctx.camera.move_time = ctx.time;
		ctx.camera.move_spd = ent.base_spd as f32 / chipcore::FPS as f32;
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
		obj.anim.anims.push(render::AnimState::MoveZ(render::MoveZ { target_z: 50.0, move_spd: 200.0 }));
	}
	else if faded {
		obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
	}
	else if matches!(kind, chipty::EntityKind::Bomb) {
		obj.anim.anims.clear();
	}
	obj.anim.unalive_after_anim = true;
}

pub fn entity_step(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	let src = ent.pos - match ent.step_dir { Some(step_dir) => step_dir.to_vec(), None => Vec2::ZERO };

	let start_pos = ent_pos(&ctx.gs, ent, src, false);
	let end_pos = ent_pos(&ctx.gs, ent, ent.pos, false);
	obj.data.pos = start_pos;

	obj.data.sprite = animated_sprite_for_ent(ent, &ctx.gs);
	obj.data.frame = 0;

	// Ensure the previous step animation is cleared...
	// See [MoveStep::animate] setting obj.pos when the animation completes.
	obj.anim.anims.clear();

	let jump_height = match ent.kind {
		chipty::EntityKind::PinkBall => 6.0,
		chipty::EntityKind::Walker => 6.0,
		_ => 0.0,
	};

	obj.anim.anims.push(render::AnimState::MoveStep(render::MoveStep {
		start_pos,
		end_pos,
		move_time: ctx.time,
		duration: ent.step_spd as f32 / chipcore::FPS as f32,
		jump_height,
	}));
	obj.anim.anims.push(render::AnimState::AnimSeq(render::SpriteAnimSeq {
		start_time: ctx.time,
		frame_count: 4, //render::sprite_frames(&resx.spritesheet_meta, obj.data.sprite),
		frame_rate: 16.0,
	}));

	// Quick hack to flatten sprites on top of walls
	// obj.data.model = if end_pos.z >= 20.0 { chipty::ModelId::FloorSprite } else { model_for_ent(ent) };

	if ehandle == ctx.gs.ps.master {
		ctx.camera.move_src = src;
		ctx.camera.move_dest = ent.pos;
		ctx.camera.move_time = ctx.time;
		ctx.camera.move_spd = ent.step_spd as f32 / chipcore::FPS as f32;
		ctx.camera.move_teleport = false;
	}
}

pub fn entity_teleport(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	// Step out of the teleport
	entity_step(ctx, ehandle);

	// When teleporting the player snap the camera
	if ehandle == ctx.gs.ps.master {
		ctx.camera.move_teleport = true;
	}
}

pub fn entity_face_dir(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	obj.data.sprite = sprite_for_ent(ent, &ctx.gs);
}

pub fn player_game_over(ctx: &mut FxState, ehandle: chipcore::EntityHandle, reason: chipcore::GameOverReason) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	// Update the player sprite
	obj.data.sprite = match reason {
		chipcore::GameOverReason::LevelComplete => chipty::SpriteId::PlayerCheer,
		chipcore::GameOverReason::Drowned => chipty::SpriteId::WaterSplash,
		chipcore::GameOverReason::Burned => chipty::SpriteId::PlayerBurned,
		chipcore::GameOverReason::Bombed => chipty::SpriteId::PlayerBurned,
		chipcore::GameOverReason::Collided => chipty::SpriteId::PlayerBurned,
		chipcore::GameOverReason::Eaten => chipty::SpriteId::PlayerBurned,
		chipcore::GameOverReason::TimeOut => chipty::SpriteId::PlayerBurned,
		chipcore::GameOverReason::NotOkay => chipty::SpriteId::PlayerBurned,
	};

	if matches!(reason, chipcore::GameOverReason::LevelComplete) {
		handlers::effect(ctx, ent.pos, render::EffectType::Fireworks);
	}
}

pub fn player_activity(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	// PlayerActivity is fired after PlayerGameOver in case of drowning and burning...
	if !ctx.gs.is_game_over() {
		entity_face_dir(ctx, ehandle);
	}
}

pub fn player_push(ctx: &mut FxState, ehandle: chipcore::EntityHandle) {
	let Some(&obj_handle) = ctx.objlookup.get(&ehandle) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };
	let Some(ent) = ctx.gs.ents.get(ehandle) else { return };

	if !matches!(ent.kind, chipty::EntityKind::Player) {
		return;
	}

	// No pushing animation in water
	let terrain = ctx.gs.field.get_terrain(ent.pos);
	if matches!(terrain, chipty::Terrain::Water) {
		return;
	}

	obj.data.sprite = match ent.face_dir.or(ent.step_dir) {
		Some(chipty::Compass::Up) => chipty::SpriteId::PlayerPushN,
		Some(chipty::Compass::Down) => chipty::SpriteId::PlayerPushS,
		Some(chipty::Compass::Left) => chipty::SpriteId::PlayerPushW,
		Some(chipty::Compass::Right) => chipty::SpriteId::PlayerPushE,
		_ => chipty::SpriteId::PlayerWalkIdle,
	};
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
		(chipty::Terrain::InvisibleWall | chipty::Terrain::HiddenWall, _) => handlers::remove_wall_mirage(ctx, pos),
		(_, chipty::Terrain::InvisibleWall | chipty::Terrain::HiddenWall) => handlers::create_wall_mirage(ctx, pos),
		_ => {}
	}
}

pub fn fire_hidden(ctx: &mut FxState, pos: Vec2i, hidden: bool) {
	let Some(&obj_handle) = ctx.fire_sprites.get(&pos) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	obj.data.visible = !hidden;
}

pub fn create_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0 - 2.0, 0.0), // Make fire appear below other sprites
			sprite: chipty::SpriteId::FireA,
			frame: 0,
			model: chipty::ModelId::Sprite,
			alpha: 1.0,
			visible: true,
			greyscale: false,
		},
		anim: render::Animation {
			anims: vec![render::AnimState::AnimLoop(render::SpriteAnimLoop {
				start_time: ctx.time + ctx.random.next_f64() * 10.0,
				frame_rate: 8.0,
			})],
			unalive_after_anim: false,
		},
	};
	let handle = ctx.render.objects.alloc();
	ctx.render.objects.insert(handle, obj);
	ctx.fire_sprites.insert(pos, handle);
}
pub fn remove_fire(ctx: &mut FxState, pos: Vec2<i32>) {
	let Some(obj_handle) = ctx.fire_sprites.remove(&pos) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	obj.anim.anims.push(render::AnimState::FadeOut(render::FadeOut { atime: 0.0 }));
	obj.anim.unalive_after_anim = true;
}

fn model_for_ent(ent: &chipcore::Entity) -> chipty::ModelId {
	match ent.kind {
		chipty::EntityKind::Block => chipty::ModelId::Wall,
		chipty::EntityKind::IceBlock => chipty::ModelId::Wall,
		chipty::EntityKind::Tank => chipty::ModelId::ReallyFlatSprite,
		chipty::EntityKind::Bug => chipty::ModelId::FlatSprite,
		chipty::EntityKind::Blob => chipty::ModelId::ReallyFlatSprite,
		chipty::EntityKind::Paramecium => chipty::ModelId::ReallyFlatSprite,
		_ => chipty::ModelId::Sprite,
	}
}

fn sprite_for_player(face_dir: Option<chipty::Compass>, terrain: chipty::Terrain) -> chipty::SpriteId {
	if matches!(terrain, chipty::Terrain::Water) {
		match face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::PlayerSwimN,
			Some(chipty::Compass::Down) => chipty::SpriteId::PlayerSwimS,
			Some(chipty::Compass::Left) => chipty::SpriteId::PlayerSwimW,
			Some(chipty::Compass::Right) => chipty::SpriteId::PlayerSwimE,
			_ => chipty::SpriteId::PlayerSwimIdle,
		}
	}
	else {
		match face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::PlayerWalkN,
			Some(chipty::Compass::Down) => chipty::SpriteId::PlayerWalkS,
			Some(chipty::Compass::Left) => chipty::SpriteId::PlayerWalkW,
			Some(chipty::Compass::Right) => chipty::SpriteId::PlayerWalkE,
			_ => chipty::SpriteId::PlayerWalkIdle,
		}
	}
}

fn sprite_for_playernpc(face_dir: Option<chipty::Compass>, terrain: chipty::Terrain) -> chipty::SpriteId {
	if matches!(terrain, chipty::Terrain::Water) {
		match face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::PlayerSwimN,
			Some(chipty::Compass::Down) => chipty::SpriteId::PlayerSwimS,
			Some(chipty::Compass::Left) => chipty::SpriteId::PlayerSwimW,
			Some(chipty::Compass::Right) => chipty::SpriteId::PlayerSwimE,
			_ => chipty::SpriteId::PlayerSwimIdle,
		}
	}
	else if matches!(terrain, chipty::Terrain::Fire) {
		chipty::SpriteId::PlayerBurned
	}
	else {
		match face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::PlayerWalkN,
			Some(chipty::Compass::Down) => chipty::SpriteId::PlayerWalkS,
			Some(chipty::Compass::Left) => chipty::SpriteId::PlayerWalkW,
			Some(chipty::Compass::Right) => chipty::SpriteId::PlayerWalkE,
			_ => chipty::SpriteId::PlayerBurned,
		}
	}
}

fn animated_sprite_for_ent(ent: &chipcore::Entity, gs: &chipcore::GameState) -> chipty::SpriteId {
	match ent.kind {
		chipty::EntityKind::Player => sprite_for_player(ent.face_dir, gs.field.get_terrain(ent.pos)),
		chipty::EntityKind::PlayerNPC => sprite_for_playernpc(ent.face_dir, gs.field.get_terrain(ent.pos)),
		chipty::EntityKind::Chip => chipty::SpriteId::Chip,
		chipty::EntityKind::Socket => chipty::SpriteId::Socket,
		chipty::EntityKind::Block => chipty::SpriteId::DirtBlock,
		chipty::EntityKind::IceBlock => chipty::SpriteId::IceBlock,
		chipty::EntityKind::Flippers => chipty::SpriteId::Flippers,
		chipty::EntityKind::FireBoots => chipty::SpriteId::FireBoots,
		chipty::EntityKind::IceSkates => chipty::SpriteId::IceSkates,
		chipty::EntityKind::SuctionBoots => chipty::SpriteId::SuctionBoots,
		chipty::EntityKind::BlueKey => chipty::SpriteId::BlueKey,
		chipty::EntityKind::RedKey => chipty::SpriteId::RedKey,
		chipty::EntityKind::GreenKey => chipty::SpriteId::GreenKey,
		chipty::EntityKind::YellowKey => chipty::SpriteId::YellowKey,
		chipty::EntityKind::Thief => chipty::SpriteId::Thief,
		chipty::EntityKind::Bug => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::BugNA,
			Some(chipty::Compass::Down) => chipty::SpriteId::BugSA,
			Some(chipty::Compass::Left) => chipty::SpriteId::BugWA,
			Some(chipty::Compass::Right) => chipty::SpriteId::BugEA,
			_ => chipty::SpriteId::BugN,
		},
		chipty::EntityKind::Tank => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::TankN,
			Some(chipty::Compass::Down) => chipty::SpriteId::TankS,
			Some(chipty::Compass::Left) => chipty::SpriteId::TankW,
			Some(chipty::Compass::Right) => chipty::SpriteId::TankE,
			_ => chipty::SpriteId::TankN,
		},
		chipty::EntityKind::PinkBall => chipty::SpriteId::PinkBall,
		chipty::EntityKind::FireBall => chipty::SpriteId::FireballA,
		chipty::EntityKind::Glider => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::GliderNA,
			Some(chipty::Compass::Down) => chipty::SpriteId::GliderSA,
			Some(chipty::Compass::Left) => chipty::SpriteId::GliderWA,
			Some(chipty::Compass::Right) => chipty::SpriteId::GliderEA,
			_ => chipty::SpriteId::GliderN,
		},
		chipty::EntityKind::Walker => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::WalkerNA,
			Some(chipty::Compass::Down) => chipty::SpriteId::WalkerSA,
			Some(chipty::Compass::Left) => chipty::SpriteId::WalkerWA,
			Some(chipty::Compass::Right) => chipty::SpriteId::WalkerEA,
			_ => chipty::SpriteId::WalkerN,
		},
		chipty::EntityKind::Teeth => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::TeethNA,
			Some(chipty::Compass::Down) => chipty::SpriteId::TeethSA,
			Some(chipty::Compass::Left) => chipty::SpriteId::TeethWA,
			Some(chipty::Compass::Right) => chipty::SpriteId::TeethEA,
			_ => chipty::SpriteId::TeethN,
		},
		chipty::EntityKind::Blob => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::BlobNA,
			Some(chipty::Compass::Down) => chipty::SpriteId::BlobSA,
			Some(chipty::Compass::Left) => chipty::SpriteId::BlobWA,
			Some(chipty::Compass::Right) => chipty::SpriteId::BlobEA,
			_ => chipty::SpriteId::Blob,
		},
		chipty::EntityKind::Paramecium => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::ParameciumNA,
			Some(chipty::Compass::Down) => chipty::SpriteId::ParameciumSA,
			Some(chipty::Compass::Left) => chipty::SpriteId::ParameciumWA,
			Some(chipty::Compass::Right) => chipty::SpriteId::ParameciumEA,
			_ => chipty::SpriteId::ParameciumN,
		},
		chipty::EntityKind::Bomb => chipty::SpriteId::BombA,
	}
}

fn sprite_for_ent(ent: &chipcore::Entity, gs: &chipcore::GameState) -> chipty::SpriteId {
	match ent.kind {
		chipty::EntityKind::Player => sprite_for_player(ent.face_dir, gs.field.get_terrain(ent.pos)),
		chipty::EntityKind::PlayerNPC => sprite_for_playernpc(ent.face_dir, gs.field.get_terrain(ent.pos)),
		chipty::EntityKind::Chip => chipty::SpriteId::Chip,
		chipty::EntityKind::Socket => chipty::SpriteId::Socket,
		chipty::EntityKind::Block => chipty::SpriteId::DirtBlock,
		chipty::EntityKind::IceBlock => chipty::SpriteId::IceBlock,
		chipty::EntityKind::Flippers => chipty::SpriteId::Flippers,
		chipty::EntityKind::FireBoots => chipty::SpriteId::FireBoots,
		chipty::EntityKind::IceSkates => chipty::SpriteId::IceSkates,
		chipty::EntityKind::SuctionBoots => chipty::SpriteId::SuctionBoots,
		chipty::EntityKind::BlueKey => chipty::SpriteId::BlueKey,
		chipty::EntityKind::RedKey => chipty::SpriteId::RedKey,
		chipty::EntityKind::GreenKey => chipty::SpriteId::GreenKey,
		chipty::EntityKind::YellowKey => chipty::SpriteId::YellowKey,
		chipty::EntityKind::Thief => chipty::SpriteId::Thief,
		chipty::EntityKind::Bug => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::BugN,
			Some(chipty::Compass::Down) => chipty::SpriteId::BugS,
			Some(chipty::Compass::Left) => chipty::SpriteId::BugW,
			Some(chipty::Compass::Right) => chipty::SpriteId::BugE,
			_ => chipty::SpriteId::BugN,
		},
		chipty::EntityKind::Tank => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::TankN,
			Some(chipty::Compass::Down) => chipty::SpriteId::TankS,
			Some(chipty::Compass::Left) => chipty::SpriteId::TankW,
			Some(chipty::Compass::Right) => chipty::SpriteId::TankE,
			_ => chipty::SpriteId::TankN,
		},
		chipty::EntityKind::PinkBall => chipty::SpriteId::PinkBall,
		chipty::EntityKind::FireBall => chipty::SpriteId::Fireball,
		chipty::EntityKind::Glider => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::GliderN,
			Some(chipty::Compass::Down) => chipty::SpriteId::GliderS,
			Some(chipty::Compass::Left) => chipty::SpriteId::GliderW,
			Some(chipty::Compass::Right) => chipty::SpriteId::GliderE,
			_ => chipty::SpriteId::GliderN,
		},
		chipty::EntityKind::Walker => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::WalkerN,
			Some(chipty::Compass::Down) => chipty::SpriteId::WalkerS,
			Some(chipty::Compass::Left) => chipty::SpriteId::WalkerW,
			Some(chipty::Compass::Right) => chipty::SpriteId::WalkerE,
			_ => chipty::SpriteId::WalkerN,
		},
		chipty::EntityKind::Teeth => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::TeethN,
			Some(chipty::Compass::Down) => chipty::SpriteId::TeethS,
			Some(chipty::Compass::Left) => chipty::SpriteId::TeethW,
			Some(chipty::Compass::Right) => chipty::SpriteId::TeethE,
			_ => chipty::SpriteId::TeethN,
		},
		chipty::EntityKind::Blob => chipty::SpriteId::Blob,
		chipty::EntityKind::Paramecium => match ent.face_dir {
			Some(chipty::Compass::Up) => chipty::SpriteId::ParameciumN,
			Some(chipty::Compass::Down) => chipty::SpriteId::ParameciumS,
			Some(chipty::Compass::Left) => chipty::SpriteId::ParameciumW,
			Some(chipty::Compass::Right) => chipty::SpriteId::ParameciumE,
			_ => chipty::SpriteId::ParameciumN,
		}
		chipty::EntityKind::Bomb => chipty::SpriteId::Bomb,
	}
}

pub fn lock_opened(ctx: &mut FxState, pos: Vec2<i32>, key: chipcore::KeyColor) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
			sprite: match key {
				chipcore::KeyColor::Red => chipty::SpriteId::RedLock,
				chipcore::KeyColor::Green => chipty::SpriteId::GreenLock,
				chipcore::KeyColor::Blue => chipty::SpriteId::BlueLock,
				chipcore::KeyColor::Yellow => chipty::SpriteId::YellowLock,
			},
			frame: 0,
			model: chipty::ModelId::Wall,
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
			sprite: chipty::SpriteId::RealBlueWall,
			frame: 0,
			model: chipty::ModelId::Wall,
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
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, -20.0),
			sprite: chipty::SpriteId::Wall,
			frame: 0,
			model: chipty::ModelId::Wall,
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
			sprite: chipty::SpriteId::Wall,
			frame: 0,
			model: chipty::ModelId::ToggleWall,
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
	ctx.toggle_walls.insert(pos, handle);
}

pub fn remove_toggle_wall(ctx: &mut FxState, pos: Vec2<i32>) {
	let Some(obj_handle) = ctx.toggle_walls.remove(&pos) else { return };
	ctx.render.objects.remove(obj_handle);
}

pub fn toggle_wall(ctx: &mut FxState, pos: Vec2i) {
	let Some(&obj_handle) = ctx.toggle_walls.get(&pos) else { return };
	let Some(obj) = ctx.render.objects.get_mut(obj_handle) else { return };

	let terrain = ctx.gs.field.get_terrain(pos);
	obj.anim.anims.clear();
	obj.anim.anims.push(render::AnimState::MoveZ(render::MoveZ {
		target_z: if matches!(terrain, chipty::Terrain::ToggleWall) { 0.0 } else { -21.0 },
		move_spd: 200.0,
	}));
}

pub fn create_wall_mirage(ctx: &mut FxState, pos: Vec2<i32>) {
	let obj = render::Object {
		data: render::ObjectData {
			pos: Vec3::new(pos.x as f32 * 32.0, pos.y as f32 * 32.0, 0.0),
			sprite: chipty::SpriteId::Wall,
			frame: 0,
			model: chipty::ModelId::Wall,
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
	ctx.mirage_walls.insert(pos, handle);
}

pub fn remove_wall_mirage(ctx: &mut FxState, pos: Vec2<i32>) {
	let Some(obj_handle) = ctx.mirage_walls.remove(&pos) else { return };

	ctx.render.objects.remove(obj_handle);
}

pub fn game_over(ctx: &mut FxState, reason: chipcore::GameOverReason) {
	ctx.game_realtime = (ctx.time - ctx.game_start_time) as f32;
	ctx.next_level_load = ctx.time + 2.0;
	ctx.game_over = Some(reason);
}

pub fn effect(ctx: &mut FxState, pos: Vec2i, ty: render::EffectType) {
	let pos = Vec3::new(pos.x as f32 * 32.0 + 16.0, pos.y as f32 * 32.0 + 16.0, 10.0);
	let start = ctx.time;
	ctx.render.effects.push(render::Effect { ty, pos, start });
}
