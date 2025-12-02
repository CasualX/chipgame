use super::*;

/// Solid flags.
#[derive(Debug)]
pub struct SolidFlags {
	pub gravel: bool,
	pub fire: bool,
	pub dirt: bool,
	pub water: bool,
	pub exit: bool,
	pub blue_fake: bool,
	pub recessed_wall: bool,
	pub keys: bool,
	pub solid_key: bool,
	pub boots: bool,
	pub chips: bool,
	pub creatures: bool,
	pub player: bool,
	pub thief: bool,
	pub hint: bool,
}

/// Checks if the entity can move in the given step direction.
///
/// If no direction is given, checks whether the given position is valid to move to.
pub fn can_move(s: &GameState, mut pos: Vec2i, step_dir: Option<Compass>, flags: &SolidFlags) -> bool {
	let mut terrain = s.field.get_terrain(pos);

	if let Some(step_dir) = step_dir {
		// Check for panels on the current terrain
		let solidf = terrain_solid_flags(terrain, flags);
		let panel = match step_dir {
			Compass::Up => THIN_WALL_N,
			Compass::Left => THIN_WALL_W,
			Compass::Down => THIN_WALL_S,
			Compass::Right => THIN_WALL_E,
		};
		// If on a solid wall, allow movement out
		if solidf != SOLID_WALL && (solidf & panel) != 0 {
			return false;
		}

		pos += step_dir.to_vec();
		terrain = s.field.get_terrain(pos);

		// Check the solid flags of the next terrain
		let panel = match step_dir {
			Compass::Up => THIN_WALL_S,
			Compass::Left => THIN_WALL_E,
			Compass::Down => THIN_WALL_N,
			Compass::Right => THIN_WALL_W,
		};
		if terrain_solid_flags(terrain, flags) & panel != 0 {
			return false;
		}
	}

	// Check if the terrain is solid
	if terrain_solid_flags(terrain, flags) == SOLID_WALL {
		return false;
	}

	for ehandle in s.qt.get(pos) {
		let Some(ent) = s.ents.get(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => flags.player,
			EntityKind::Chip => flags.chips,
			EntityKind::Socket => true,
			EntityKind::Block => true,
			EntityKind::IceBlock => true,
			EntityKind::Flippers => flags.boots,
			EntityKind::FireBoots => flags.boots,
			EntityKind::IceSkates => flags.boots,
			EntityKind::SuctionBoots => flags.boots,
			EntityKind::BlueKey => flags.keys,
			EntityKind::RedKey => flags.keys,
			EntityKind::GreenKey => flags.solid_key,
			EntityKind::YellowKey => flags.solid_key,
			EntityKind::Thief => flags.thief,
			EntityKind::Bomb => false,
			EntityKind::Bug => flags.creatures,
			EntityKind::FireBall => flags.creatures,
			EntityKind::PinkBall => flags.creatures,
			EntityKind::Tank => flags.creatures,
			EntityKind::Glider => flags.creatures,
			EntityKind::Teeth => flags.creatures,
			EntityKind::Walker => flags.creatures,
			EntityKind::Blob => flags.creatures,
			EntityKind::Paramecium => flags.creatures,
		};
		if solid {
			return false;
		}
	}

	return true;
}

/// Try to unlock a lock.
fn try_unlock(s: &mut GameState, pos: Vec2i, key: KeyColor) -> bool {
	if s.ps.keys[key as usize] <= 0 {
		return false;
	}
	s.set_terrain(pos, Terrain::Floor);
	if !matches!(key, KeyColor::Green) {
		s.ps.keys[key as usize] -= 1;
	}
	s.events.fire(GameEvent::LockOpened { pos, key });
	s.events.fire(GameEvent::SoundFx { sound: SoundFx::LockOpened });
	return true;
}

/// Tries to move the entity in the given step direction.
pub fn try_move(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	ent.flags &= !EF_NEW_POS;

	// Template entities cannot be moved
	if ent.flags & EF_TEMPLATE != 0 {
		return false;
	}

	let mover_kind = ent.kind;
	let dev_wtw = matches!(mover_kind, EntityKind::Player) && s.ps.dev_wtw;

	// Kinda tricky workaround: Trapped entities can move if they have been released from the trap
	if ent.base_spd == 0 || ent.is_trapped() {
		return false;
	}

	let new_pos = ent.pos + step_dir.to_vec();
	let from_terrain = s.field.get_terrain(ent.pos);
	let mut to_terrain = s.field.get_terrain(new_pos);

	// Set the entity's step speed based on the terrain
	if matches!(from_terrain, Terrain::ForceW | Terrain::ForceE | Terrain::ForceN | Terrain::ForceS | Terrain::ForceRandom) {
		if matches!(mover_kind, EntityKind::Player) {
			if s.ps.suction_boots {
				ent.step_spd = ent.base_spd;
				ps_activity(s, ent.handle, PlayerActivity::ForceWalking);
			}
			else {
				ent.step_spd = cmp::max(1, ent.base_spd / 2);
				ps_activity(s, ent.handle, PlayerActivity::ForceSliding);
			}
		}
		else {
			ent.step_spd = cmp::max(1, ent.base_spd / 2);
		}
	}
	else if matches!(from_terrain, Terrain::Ice | Terrain::IceNE | Terrain::IceSE | Terrain::IceNW | Terrain::IceSW) {
		if matches!(mover_kind, EntityKind::Player) {
			if s.ps.ice_skates {
				ent.step_spd = ent.base_spd;
				ps_activity(s, ent.handle, PlayerActivity::IceSkating);
			}
			else {
				ent.step_spd = cmp::max(1, ent.base_spd / 2);
				ps_activity(s, ent.handle, PlayerActivity::IceSliding);
			}
		}
		else {
			ent.step_spd = cmp::max(1, ent.base_spd / 2);
		}
	}
	else {
		ent.step_spd = ent.base_spd;
	}

	if !dev_wtw {
		// Check for panels on the current terrain
		let solidf = terrain_solid_flags(from_terrain, &ent.data.flags);
		let panel = match step_dir {
			Compass::Up => THIN_WALL_N,
			Compass::Left => THIN_WALL_W,
			Compass::Down => THIN_WALL_S,
			Compass::Right => THIN_WALL_E,
		};
		// Allow movement out of solid tiles (e.g. toggle walls)
		// Otherwise block movement when a panel blocks exit and disable flicks
		if solidf != SOLID_WALL && (solidf & panel) != 0 {
			//flick(s, ent.kind, &new_pos, step_dir);
			return false;
		}

		// Player specific interactions with the terrain
		if matches!(mover_kind, EntityKind::Player) {
			let solid = match to_terrain {
				Terrain::BlueLock => !try_unlock(s, new_pos, KeyColor::Blue),
				Terrain::RedLock => !try_unlock(s, new_pos, KeyColor::Red),
				Terrain::GreenLock => !try_unlock(s, new_pos, KeyColor::Green),
				Terrain::YellowLock => !try_unlock(s, new_pos, KeyColor::Yellow),
				Terrain::RealBlueWall => {
					s.set_terrain(new_pos, Terrain::Wall);
					true
				}
				Terrain::FakeBlueWall => {
					s.set_terrain(new_pos, Terrain::Floor);
					s.events.fire(GameEvent::SoundFx { sound: SoundFx::BlueWallCleared });
					false
				}
				Terrain::HiddenWall => {
					s.set_terrain(new_pos, Terrain::Wall);
					true
				}
				_ => false,
			};
			if solid {
				flick(s, ent.kind, &new_pos, step_dir);
				return false;
			}
		}

		// Update the terrain after potential changes
		to_terrain = s.field.get_terrain(new_pos);

		// Check the solid flags of the next terrain
		let panel = match step_dir {
			Compass::Up => THIN_WALL_S,
			Compass::Left => THIN_WALL_E,
			Compass::Down => THIN_WALL_N,
			Compass::Right => THIN_WALL_W,
		};
		if terrain_solid_flags(to_terrain, &ent.data.flags) & panel != 0 {
			flick(s, ent.kind, &new_pos, step_dir);
			return false;
		}
	}

	let flags = &ent.data.flags;
	let pusher = ent.kind;
	for ehandle in s.qt.get(new_pos) {
		let Some(mut ent) = s.ents.take(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => {
				if matches!(mover_kind, EntityKind::Player) {
					!try_push_block(s, &mut ent, step_dir)
				}
				else {
					flags.player
				}
			}
			EntityKind::Chip => flags.chips,
			EntityKind::Socket => {
				if matches!(mover_kind, EntityKind::Player) && s.ps.chips >= s.field.required_chips {
					ent.flags |= EF_REMOVE;
					s.events.fire(GameEvent::SocketFilled { pos: ent.pos });
					s.events.fire(GameEvent::SoundFx { sound: SoundFx::SocketOpened });
					false
				}
				else {
					!matches!(mover_kind, EntityKind::Bomb)
				}
			}
			EntityKind::Block => {
				if matches!(mover_kind, EntityKind::Player) && try_push_block(s, &mut ent, step_dir) {
					s.update_hidden_flag(ent.pos, true);
					s.update_hidden_flag(ent.pos - step_dir.to_vec(), false);
					s.events.fire(GameEvent::BlockPush { entity: ent.handle });
					s.events.fire(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
					false
				}
				else {
					!matches!(mover_kind, EntityKind::Bomb)
				}
			}
			EntityKind::IceBlock => {
				let allowed_pusher = matches!(pusher, EntityKind::Player | EntityKind::IceBlock | EntityKind::Teeth | EntityKind::Tank);
				if allowed_pusher && try_push_block(s, &mut ent, step_dir) {
					s.update_hidden_flag(ent.pos, true);
					s.update_hidden_flag(ent.pos - step_dir.to_vec(), false);
					s.events.fire(GameEvent::BlockPush { entity: ent.handle });
					if matches!(mover_kind, EntityKind::Player) { // Only play sound if player is pushing the ice block
						s.events.fire(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
					}
					false
				}
				else {
					!matches!(mover_kind, EntityKind::Bomb)
				}
			}
			EntityKind::Flippers => flags.boots,
			EntityKind::FireBoots => flags.boots,
			EntityKind::IceSkates => flags.boots,
			EntityKind::SuctionBoots => flags.boots,
			EntityKind::BlueKey => flags.keys,
			EntityKind::RedKey => flags.keys,
			EntityKind::GreenKey => flags.solid_key,
			EntityKind::YellowKey => flags.solid_key,
			EntityKind::Thief => flags.thief,
			EntityKind::Bomb => false,
			EntityKind::Bug => flags.creatures,
			EntityKind::FireBall => flags.creatures,
			EntityKind::PinkBall => flags.creatures,
			EntityKind::Tank => flags.creatures,
			EntityKind::Glider => flags.creatures,
			EntityKind::Teeth => flags.creatures,
			EntityKind::Walker => flags.creatures,
			EntityKind::Blob => flags.creatures,
			EntityKind::Paramecium => flags.creatures,
		};
		s.ents.put(ent);
		if solid {
			return false;
		}
	}

	// Slap code
	if matches!(mover_kind, EntityKind::Player) && s.ps.last_step_dir == Some(step_dir) {
		if matches!(step_dir, Compass::Up | Compass::Down) {
			if s.ps.inbuf.is_any_dir(Compass::Left) {
				slap(s, ent.pos, Compass::Left);
			}
			if s.ps.inbuf.is_any_dir(Compass::Right) {
				slap(s, ent.pos, Compass::Right);
			}
		}
		if matches!(step_dir, Compass::Left | Compass::Right) {
			if s.ps.inbuf.is_any_dir(Compass::Up) {
				slap(s, ent.pos, Compass::Up);
			}
			if s.ps.inbuf.is_any_dir(Compass::Down) {
				slap(s, ent.pos, Compass::Down);
			}
		}
	}

	s.qt.update(ent.handle, ent.pos, new_pos);

	ent.face_dir = Some(step_dir);
	ent.step_dir = Some(step_dir);
	ent.step_time = s.time;
	ent.pos = new_pos;
	ent.flags |= EF_NEW_POS;
	ent.flags &= !(EF_RELEASED | EF_TRAPPED);

	// Retain momentum when entity lands on a Trap
	if !matches!(to_terrain, Terrain::BearTrap) {
		ent.flags &= !EF_MOMENTUM;
	}

	if matches!(mover_kind, EntityKind::Player) {
		s.ps.steps += 1;
	}
	s.events.fire(GameEvent::EntityTurn { entity: ent.handle });
	s.events.fire(GameEvent::EntityStep { entity: ent.handle });
	return true;
}

/// Tries to push a Block in the given push direction.
fn try_push_block(s: &mut GameState, ent: &mut Entity, push_dir: Compass) -> bool {
	// The Player movement happens before the Block's movement code
	// So preemptively move the Block by terrain if possible

	// This fixes bumping blocks in the following scenarios:
	// Player -> Block -> Ice corner
	// Player -> Block -> ForceN -> Wall
	// The Block should slide on the ice corner (getting out of the player's way)
	// instead of blocking the player because it hit a wall

	// Oh and hardcode fast terrain speed for checking terrain movement
	// Otherwise the following scenario causes a bump on the 2nd ice corner:
	// Player -> Block -> Ice corner -> Floor -> Ice corner
	// When the Block is moved from the Floor tile it has its base speed while on the Ice corner

	if s.time >= ent.step_time + ent.base_spd / 2 && try_terrain_move(s, ent, ent.step_dir) {
		// Returns true if the Block was actually moved out of the way
		if s.time == ent.step_time && ent.flags & EF_NEW_POS != 0 {
			return true;
		}
	}

	// Otherwise, try to push the Block manually
	// Note that we ignore the Block's step timer here to allow pushing off of ice or force floors
	try_move(s, ent, push_dir)
}

/// To flick a Block is to push it off a tile that Chip cannot enter.
fn flick(s: &mut GameState, pusher: EntityKind, &new_pos: &Vec2i, step_dir: Compass) {
	let allowed_block_pusher = matches!(pusher, EntityKind::Player);
	let allowed_iceblock_pusher = matches!(pusher, EntityKind::Player | EntityKind::IceBlock | EntityKind::Teeth | EntityKind::Tank);

	if !(allowed_block_pusher || allowed_iceblock_pusher) {
		return;
	}

	for ehandle in s.qt.get(new_pos) {
		let Some(mut ent) = s.ents.take(ehandle) else { continue };

		let allowed_pusher =
			matches!(ent.kind, EntityKind::Block) && allowed_block_pusher ||
			matches!(ent.kind, EntityKind::IceBlock) && allowed_iceblock_pusher;

		if allowed_pusher {
			if try_move(s, &mut ent, step_dir) {
				s.update_hidden_flag(ent.pos, true);
				s.update_hidden_flag(ent.pos - step_dir.to_vec(), false);
				s.events.fire(GameEvent::BlockPush { entity: ent.handle });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
			}
		}

		s.ents.put(ent);
	}
}

fn slap(s: &mut GameState, player_pos: Vec2i, slap_dir: Compass) {
	let pos = player_pos + slap_dir.to_vec();

	// Slap the terrain
	match s.field.get_terrain(pos) {
		Terrain::RealBlueWall => {
			s.set_terrain(pos, Terrain::Wall);
		}
		Terrain::HiddenWall => {
			s.set_terrain(pos, Terrain::Wall);
		}
		_ => {}
	}

	// Slap blocks
	for ehandle in s.qt.get(pos) {
		let Some(mut ent) = s.ents.take(ehandle) else { continue };

		if matches!(ent.kind, EntityKind::Block | EntityKind::IceBlock) {
			if try_move(s, &mut ent, slap_dir) {
				s.update_hidden_flag(ent.pos, true);
				s.update_hidden_flag(ent.pos - slap_dir.to_vec(), false);
				s.events.fire(GameEvent::BlockPush { entity: ent.handle });
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
			}
		}

		s.ents.put(ent);
	}
}

/// Tries to move the entity according to the terrain effects.
/// Returns true if the entity is under the influence of terrain movement (not necessarily moved).
pub fn try_terrain_move(s: &mut GameState, ent: &mut Entity, step_dir: Option<Compass>) -> bool {
	// Entity is trapped, cannot move or turn
	if ent.is_trapped() {
		return true;
	}

	match s.field.get_terrain(ent.pos) {
		// When an entity is released from a trap, push it out of the trap in the direction it entered
		// CC1: Level 109 - Torturechamber
		Terrain::BearTrap => {
			if ent.flags & EF_MOMENTUM != 0 {
				if let Some(step_dir) = ent.step_dir {
					if try_move(s, ent, step_dir) {
						return true;
					}
				}
			}
			return false;
		}
		Terrain::Ice => match step_dir {
			Some(dir) => !try_move(s, ent, dir) && try_move(s, ent, dir.turn_around()),
			None => return false,
		}
		Terrain::IceNW => match step_dir {
			Some(Compass::Up) => !try_move(s, ent, Compass::Right) && try_move(s, ent, Compass::Down),
			Some(Compass::Left) => !try_move(s, ent, Compass::Down) && try_move(s, ent, Compass::Right),
			Some(Compass::Down) => !try_move(s, ent, Compass::Down) && try_move(s, ent, Compass::Right),
			Some(Compass::Right) => !try_move(s, ent, Compass::Right) && try_move(s, ent, Compass::Down),
			_ => return false,
		}
		Terrain::IceNE => match step_dir {
			Some(Compass::Up) => !try_move(s, ent, Compass::Left) && try_move(s, ent, Compass::Down),
			Some(Compass::Left) => !try_move(s, ent, Compass::Left) && try_move(s, ent, Compass::Down),
			Some(Compass::Down) => !try_move(s, ent, Compass::Down) && try_move(s, ent, Compass::Left),
			Some(Compass::Right) => !try_move(s, ent, Compass::Down) && try_move(s, ent, Compass::Left),
			_ => return false,
		}
		Terrain::IceSE => match step_dir {
			Some(Compass::Up) => !try_move(s, ent, Compass::Up) && try_move(s, ent, Compass::Left),
			Some(Compass::Left) => !try_move(s, ent, Compass::Left) && try_move(s, ent, Compass::Up),
			Some(Compass::Down) => !try_move(s, ent, Compass::Left) && try_move(s, ent, Compass::Up),
			Some(Compass::Right) => !try_move(s, ent, Compass::Up) && try_move(s, ent, Compass::Left),
			_ => return false,
		}
		Terrain::IceSW => match step_dir {
			Some(Compass::Up) => !try_move(s, ent, Compass::Up) && try_move(s, ent, Compass::Right),
			Some(Compass::Left) => !try_move(s, ent, Compass::Up) && try_move(s, ent, Compass::Right),
			Some(Compass::Down) => !try_move(s, ent, Compass::Right) && try_move(s, ent, Compass::Up),
			Some(Compass::Right) => !try_move(s, ent, Compass::Right) && try_move(s, ent, Compass::Up),
			_ => return false,
		}
		Terrain::ForceN => try_move(s, ent, Compass::Up),
		Terrain::ForceW => try_move(s, ent, Compass::Left),
		Terrain::ForceS => try_move(s, ent, Compass::Down),
		Terrain::ForceE => try_move(s, ent, Compass::Right),
		Terrain::ForceRandom => { let dir = s.rand.next(); try_move(s, ent, dir) },
		Terrain::Teleport => match step_dir {
			Some(step_dir) => teleport(s, ent, step_dir), // If this fails the entity gets softlocked, player is not affected
			None => false,
		},
		_ => return false,
	};
	ent.flags |= EF_MOMENTUM;
	return true;
}

/// Teleports the entity to the destination of a teleporter.
pub fn teleport(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	let old_pos = ent.pos;
	let mut teleported;
	loop {
		// Find the teleport connection
		let Some(conn) = s.field.find_conn_by_src(ent.pos) else { return false };
		// Teleport the entity
		s.qt.update(ent.handle, ent.pos, conn.dest);
		ent.pos = conn.dest;
		teleported = ent.pos != old_pos;
		// Force the entity to move out of the teleporter
		if try_move(s, ent, step_dir) {
			break;
		}
		// Reflect the entity back if they're softlocked
		// This happens when all destinations are blocked (including the source)
		if old_pos == ent.pos {
			// CCLP3: level 50 bug fix - only reflect player entities, requires a block to be softlocked on the teleporter
			if matches!(ent.kind, EntityKind::Player) && !try_move(s, ent, step_dir.turn_around()) {
				return false;
			}
			break;
		}
	}

	// Teleport the entity if they actually moved
	if teleported {
		s.events.fire(GameEvent::EntityTeleport { entity: ent.handle });
	}
	// Play sound only for player to avoid cacophony
	if matches!(ent.kind, EntityKind::Player) {
		s.events.fire(GameEvent::SoundFx { sound: SoundFx::Teleporting });
	}
	return true;
}

const SOLID_WALL: u8 = 0xf;
const THIN_WALL_N: u8 = 0x1;
const THIN_WALL_E: u8 = 0x2;
const THIN_WALL_S: u8 = 0x4;
const THIN_WALL_W: u8 = 0x8;

fn terrain_solid_flags(terrain: Terrain, flags: &SolidFlags) -> u8 {
	match terrain {
		Terrain::Blank => 0,
		Terrain::Floor => 0,
		Terrain::Wall => SOLID_WALL,
		Terrain::Socket => SOLID_WALL,
		Terrain::BlueLock => SOLID_WALL,
		Terrain::RedLock => SOLID_WALL,
		Terrain::GreenLock => SOLID_WALL,
		Terrain::YellowLock => SOLID_WALL,
		Terrain::Hint => 0, //if flags.hint { SOLID_WALL } else { 0 },
		Terrain::Exit => if flags.exit { SOLID_WALL } else { 0 },
		Terrain::FakeExit => 0,
		Terrain::Water => if flags.water { SOLID_WALL } else { 0 },
		Terrain::WaterHazard => SOLID_WALL,
		Terrain::Fire => if flags.fire { SOLID_WALL } else { 0 },
		Terrain::Dirt => if flags.dirt { SOLID_WALL } else { 0 },
		Terrain::DirtBlock => SOLID_WALL,
		Terrain::Gravel => if flags.gravel { SOLID_WALL } else { 0 },
		Terrain::Ice => 0,
		Terrain::IceNW => THIN_WALL_N | THIN_WALL_W,
		Terrain::IceNE => THIN_WALL_N | THIN_WALL_E,
		Terrain::IceSW => THIN_WALL_S | THIN_WALL_W,
		Terrain::IceSE => THIN_WALL_S | THIN_WALL_E,
		Terrain::ForceN => 0,
		Terrain::ForceW => 0,
		Terrain::ForceS => 0,
		Terrain::ForceE => 0,
		Terrain::ForceRandom => 0,
		Terrain::CloneMachine => SOLID_WALL,
		Terrain::CloneBlockN => SOLID_WALL,
		Terrain::CloneBlockW => SOLID_WALL,
		Terrain::CloneBlockS => SOLID_WALL,
		Terrain::CloneBlockE => SOLID_WALL,
		Terrain::ToggleFloor => 0,
		Terrain::ToggleWall => SOLID_WALL,
		Terrain::ThinWallN => THIN_WALL_N,
		Terrain::ThinWallW => THIN_WALL_W,
		Terrain::ThinWallS => THIN_WALL_S,
		Terrain::ThinWallE => THIN_WALL_E,
		Terrain::ThinWallSE => THIN_WALL_S | THIN_WALL_E,
		Terrain::HiddenWall => SOLID_WALL,
		Terrain::InvisibleWall => SOLID_WALL,
		Terrain::RealBlueWall => SOLID_WALL,
		Terrain::FakeBlueWall => if flags.blue_fake { SOLID_WALL } else { 0 },
		Terrain::GreenButton => 0,
		Terrain::RedButton => 0,
		Terrain::BrownButton => 0,
		Terrain::BlueButton => 0,
		Terrain::Teleport => 0,
		Terrain::BearTrap => 0,
		Terrain::RecessedWall => if flags.recessed_wall { SOLID_WALL } else { 0 },
	}
}
