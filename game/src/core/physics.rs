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
	pub pickup: bool,
	pub creature: bool,
	pub player: bool,
	pub thief: bool,
}

/// Checks if the entity can move in the given step direction.
///
/// If no direction is given, checks whether the given position is valid to move to.
pub fn can_move(s: &GameState, mut pos: Vec2i, step_dir: Option<Compass>, flags: &SolidFlags) -> bool {
	let mut terrain = s.field.get_terrain(pos);

	if let Some(step_dir) = step_dir {
		// Check for panels on the current terrain
		let solidf = terrain.solid_flags();
		let panel = match step_dir {
			Compass::Up => PANEL_N,
			Compass::Left => PANEL_W,
			Compass::Down => PANEL_S,
			Compass::Right => PANEL_E,
		};
		// If on a solid wall, allow movement out
		if solidf != SOLID_WALL && (solidf & panel) != 0 {
			return false;
		}

		pos += step_dir.to_vec();
		terrain = s.field.get_terrain(pos);

		// Check the solid flags of the next terrain
		let panel = match step_dir {
			Compass::Up => PANEL_S,
			Compass::Left => PANEL_E,
			Compass::Down => PANEL_N,
			Compass::Right => PANEL_W,
		};
		if terrain.solid_flags() & panel != 0 {
			return false;
		}
	}

	// Check if the terrain is solid
	if terrain.solid_flags() == SOLID_WALL {
		return false;
	}

	// Check if allowed to move on certain terrains
	if flags.gravel && matches!(terrain, Terrain::Gravel) {
		return false;
	}
	if flags.fire && matches!(terrain, Terrain::Fire) {
		return false;
	}
	if flags.dirt && matches!(terrain, Terrain::Dirt) {
		return false;
	}
	if flags.exit && matches!(terrain, Terrain::Exit) {
		return false;
	}
	if flags.water && matches!(terrain, Terrain::Water) {
		return false;
	}
	if flags.blue_fake && matches!(terrain, Terrain::BlueFake) {
		return false;
	}

	for ehandle in s.qt.get(pos) {
		let Some(ent) = s.ents.get(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => flags.player,
			EntityKind::Chip => flags.pickup,
			EntityKind::Socket => true,
			EntityKind::Block => true,
			EntityKind::Flippers => flags.pickup,
			EntityKind::FireBoots => flags.pickup,
			EntityKind::IceSkates => flags.pickup,
			EntityKind::SuctionBoots => flags.pickup,
			EntityKind::BlueKey => flags.pickup,
			EntityKind::RedKey => flags.pickup,
			EntityKind::GreenKey => flags.pickup,
			EntityKind::YellowKey => flags.pickup,
			EntityKind::Thief => flags.thief,
			EntityKind::Bomb => false,
			EntityKind::Bug => flags.creature,
			EntityKind::FireBall => flags.creature,
			EntityKind::PinkBall => flags.creature,
			EntityKind::Tank => flags.creature,
			EntityKind::Glider => flags.creature,
			EntityKind::Teeth => flags.creature,
			EntityKind::Walker => flags.creature,
			EntityKind::Blob => flags.creature,
			EntityKind::Paramecium => flags.creature,
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
	s.field.set_terrain(pos, Terrain::Floor);
	if !matches!(key, KeyColor::Green) {
		s.ps.keys[key as usize] -= 1;
	}
	s.events.push(GameEvent::LockOpened { pos, key });
	s.events.push(GameEvent::SoundFx { sound: SoundFx::LockOpened });
	return true;
}

/// Tries to move the entity in the given step direction.
pub fn try_move(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	let is_player = matches!(ent.kind, EntityKind::Player);
	let dev_wtw = is_player && s.ps.dev_wtw;

	if ent.base_spd == 0 || ent.trapped {
		return false;
	}

	let new_pos = ent.pos + step_dir.to_vec();
	let from_terrain = s.field.get_terrain(ent.pos);

	if !dev_wtw {
		// Check for panels on the current terrain
		let solidf = from_terrain.solid_flags();
		let panel = match step_dir {
			Compass::Up => PANEL_N,
			Compass::Left => PANEL_W,
			Compass::Down => PANEL_S,
			Compass::Right => PANEL_E,
		};
		// If on a solid wall, allow movement out
		if solidf != SOLID_WALL && (solidf & panel) != 0 {
			return false;
		}

		// Player specific interactions with the terrain
		if is_player {
			let solid = match s.field.get_terrain(new_pos) {
				Terrain::BlueLock => !try_unlock(s, new_pos, KeyColor::Blue),
				Terrain::RedLock => !try_unlock(s, new_pos, KeyColor::Red),
				Terrain::GreenLock => !try_unlock(s, new_pos, KeyColor::Green),
				Terrain::YellowLock => !try_unlock(s, new_pos, KeyColor::Yellow),
				Terrain::BlueWall => {
					s.field.set_terrain(new_pos, Terrain::Wall);
					s.events.push(GameEvent::TerrainUpdated { pos: new_pos, old: Terrain::BlueWall, new: Terrain::Wall });
					true
				}
				Terrain::BlueFake => {
					s.field.set_terrain(new_pos, Terrain::Floor);
					s.events.push(GameEvent::TerrainUpdated { pos: new_pos, old: Terrain::BlueFake, new: Terrain::Floor });
					s.events.push(GameEvent::SoundFx { sound: SoundFx::BlueWallCleared });
					false
				}
				Terrain::HiddenWall => {
					s.field.set_terrain(new_pos, Terrain::HiddenWallRevealed);
					s.events.push(GameEvent::TerrainUpdated { pos: new_pos, old: Terrain::HiddenWall, new: Terrain::HiddenWallRevealed });
					true
				}
				_ => false,
			};
			if solid {
				return false;
			}
		}

		let to_terrain = s.field.get_terrain(new_pos);

		// Check the solid flags of the next terrain
		let panel = match step_dir {
			Compass::Up => PANEL_S,
			Compass::Left => PANEL_E,
			Compass::Down => PANEL_N,
			Compass::Right => PANEL_W,
		};
		if to_terrain.solid_flags() & panel != 0 {
			return false;
		}

		// Check if the terrain is solid
		if to_terrain.solid_flags() == SOLID_WALL {
			return false;
		}

		// Check if allowed to move on certain terrains
		let flags = &ent.data.flags;
		match to_terrain {
			Terrain::Gravel if flags.gravel => return false,
			Terrain::Fire if flags.fire => return false,
			Terrain::Dirt if flags.dirt => return false,
			Terrain::Exit if flags.exit => return false,
			Terrain::Water if flags.water => return false,
			Terrain::BlueFake if flags.blue_fake => return false,

			_ => (),
		}
	}

	let flags = &ent.data.flags;
	for ehandle in s.qt.get(new_pos) {
		let Some(mut ent) = s.ents.take(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => flags.player,
			EntityKind::Chip => flags.pickup,
			EntityKind::Socket => {
				if is_player && s.ps.chips >= s.field.chips {
					ent.remove = true;
					s.events.push(GameEvent::SocketFilled { pos: ent.pos });
					s.events.push(GameEvent::SoundFx { sound: SoundFx::SocketOpened });
					false
				}
				else {
					true
				}
			}
			EntityKind::Block => {
				if is_player && try_move(s, &mut ent, step_dir) {
					update_hidden_flag(s, ent.pos);
					update_hidden_flag(s, ent.pos - step_dir.to_vec());
					s.events.push(GameEvent::BlockPush { entity: ent.handle });
					s.events.push(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
					false
				}
				else {
					true
				}
			}
			EntityKind::Flippers => flags.pickup,
			EntityKind::FireBoots => flags.pickup,
			EntityKind::IceSkates => flags.pickup,
			EntityKind::SuctionBoots => flags.pickup,
			EntityKind::BlueKey => flags.pickup,
			EntityKind::RedKey => flags.pickup,
			EntityKind::GreenKey => flags.pickup,
			EntityKind::YellowKey => flags.pickup,
			EntityKind::Thief => flags.thief,
			EntityKind::Bomb => false,
			EntityKind::Bug => flags.creature,
			EntityKind::FireBall => flags.creature,
			EntityKind::PinkBall => flags.creature,
			EntityKind::Tank => flags.creature,
			EntityKind::Glider => flags.creature,
			EntityKind::Teeth => flags.creature,
			EntityKind::Walker => flags.creature,
			EntityKind::Blob => flags.creature,
			EntityKind::Paramecium => flags.creature,
		};
		s.ents.put(ent);
		if solid {
			return false;
		}
	}

	// Set the entity's step speed based on the terrain
	let has_suction_boots = is_player && s.ps.suction_boots;
	let has_ice_skates = is_player && s.ps.ice_skates;
	ent.step_spd = match from_terrain {
		Terrain::ForceW | Terrain::ForceE | Terrain::ForceN | Terrain::ForceS | Terrain::ForceRandom if !has_suction_boots => {
			if is_player {
				ps_activity(s, PlayerActivity::Sliding);
			}
			cmp::max(1, ent.base_spd / 2)
		},
		Terrain::Ice | Terrain::IceNE | Terrain::IceSE | Terrain::IceNW | Terrain::IceSW if !has_ice_skates => {
			if is_player {
				ps_activity(s, PlayerActivity::Sliding);
			}
			cmp::max(1, ent.base_spd / 2)
		},
		_ => ent.base_spd,
	};

	s.qt.update(ent.handle, ent.pos, new_pos);

	ent.face_dir = Some(step_dir);
	ent.step_dir = Some(step_dir);
	ent.step_time = s.time;
	ent.pos = new_pos;
	ent.has_moved = true;

	if is_player {
		s.ps.steps += 1;
	}
	s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
	s.events.push(GameEvent::EntityStep { entity: ent.handle });
	return true;
}

/// Teleports the entity to the destination of a teleporter.
pub fn teleport(s: &mut GameState, ent: &mut Entity, step_dir: Compass) {
	let old_pos = ent.pos;
	loop {
		// Find the teleport connection
		let Some(conn) = s.field.find_conn_by_src(ent.pos) else { return };
		// Teleport the entity
		s.qt.update(ent.handle, ent.pos, conn.dest);
		ent.pos = conn.dest;
		// Force the entity to move out of the teleporter
		if try_move(s, ent, step_dir) {
			break;
		}
		// Do nothing if no valid teleport exit is found
		if old_pos == ent.pos {
			return;
		}
	}

	// Teleport the entity
	s.events.push(GameEvent::EntityTeleport { entity: ent.handle });
	if matches!(ent.kind, EntityKind::Player) {
		s.events.push(GameEvent::SoundFx { sound: SoundFx::Teleporting });
	}
}
