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
	pub boots: bool,
	pub chips: bool,
	pub creatures: bool,
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
		if terrain.solid_flags() & panel != 0 {
			return false;
		}
	}

	// Check if the terrain is solid
	if terrain.solid_flags() == SOLID_WALL {
		return false;
	}

	// Check if allowed to move on certain terrains
	match terrain {
		Terrain::Gravel if flags.gravel => return false,
		Terrain::Fire if flags.fire => return false,
		Terrain::Dirt if flags.dirt => return false,
		Terrain::Exit if flags.exit => return false,
		Terrain::Water if flags.water => return false,
		Terrain::FakeBlueWall if flags.blue_fake => return false,
		Terrain::RecessedWall if flags.recessed_wall => return false,
		_ => (),
	}

	for ehandle in s.qt.get(pos) {
		let Some(ent) = s.ents.get(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => flags.player,
			EntityKind::Chip => flags.chips,
			EntityKind::Socket => true,
			EntityKind::Block => true,
			EntityKind::Flippers => flags.boots,
			EntityKind::FireBoots => flags.boots,
			EntityKind::IceSkates => flags.boots,
			EntityKind::SuctionBoots => flags.boots,
			EntityKind::BlueKey => flags.keys,
			EntityKind::RedKey => flags.keys,
			EntityKind::GreenKey => flags.keys,
			EntityKind::YellowKey => flags.keys,
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
	s.events.push(GameEvent::LockOpened { pos, key });
	s.events.push(GameEvent::SoundFx { sound: SoundFx::LockOpened });
	return true;
}

/// Tries to move the entity in the given step direction.
pub fn try_move(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	ent.flags &= !EF_NEW_POS;

	// Template entities cannot be moved
	if ent.flags & EF_TEMPLATE != 0 {
		return false;
	}

	let is_player = matches!(ent.kind, EntityKind::Player);
	let dev_wtw = is_player && s.ps.dev_wtw;

	if ent.base_spd == 0 || ent.flags & EF_TRAPPED != 0 {
		return false;
	}

	let new_pos = ent.pos + step_dir.to_vec();
	let from_terrain = s.field.get_terrain(ent.pos);

	if !dev_wtw {
		// Check for panels on the current terrain
		let solidf = from_terrain.solid_flags();
		let panel = match step_dir {
			Compass::Up => THIN_WALL_N,
			Compass::Left => THIN_WALL_W,
			Compass::Down => THIN_WALL_S,
			Compass::Right => THIN_WALL_E,
		};
		// If on a solid wall, allow movement out
		if solidf != SOLID_WALL && (solidf & panel) != 0 {
			flick(s, &new_pos, step_dir);
			return false;
		}

		// Player specific interactions with the terrain
		if is_player {
			let solid = match s.field.get_terrain(new_pos) {
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
					s.events.push(GameEvent::SoundFx { sound: SoundFx::BlueWallCleared });
					false
				}
				Terrain::HiddenWall => {
					s.set_terrain(new_pos, Terrain::Wall);
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
			Compass::Up => THIN_WALL_S,
			Compass::Left => THIN_WALL_E,
			Compass::Down => THIN_WALL_N,
			Compass::Right => THIN_WALL_W,
		};
		if to_terrain.solid_flags() & panel != 0 {
			flick(s, &new_pos, step_dir);
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
			Terrain::FakeBlueWall if flags.blue_fake => return false,
			Terrain::RecessedWall if flags.recessed_wall => return false,
			_ => (),
		}
	}

	let flags = &ent.data.flags;
	for ehandle in s.qt.get(new_pos) {
		let Some(mut ent) = s.ents.take(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => flags.player,
			EntityKind::Chip => flags.chips,
			EntityKind::Socket => {
				if is_player && s.ps.chips >= s.field.required_chips {
					ent.flags |= EF_REMOVE;
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
					s.update_hidden_flag(ent.pos);
					s.update_hidden_flag(ent.pos - step_dir.to_vec());
					s.events.push(GameEvent::BlockPush { entity: ent.handle });
					s.events.push(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
					false
				}
				else {
					true
				}
			}
			EntityKind::Flippers => flags.boots,
			EntityKind::FireBoots => flags.boots,
			EntityKind::IceSkates => flags.boots,
			EntityKind::SuctionBoots => flags.boots,
			EntityKind::BlueKey => flags.keys,
			EntityKind::RedKey => flags.keys,
			EntityKind::GreenKey => flags.keys,
			EntityKind::YellowKey => flags.keys,
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
	ent.flags |= EF_NEW_POS;

	// Retain momentum when entity lands on a Trap
	if !matches!(s.field.get_terrain(new_pos), Terrain::BearTrap) {
		ent.flags &= !EF_MOMENTUM;
	}

	if is_player {
		s.ps.steps += 1;
	}
	s.events.push(GameEvent::EntityTurn { entity: ent.handle });
	s.events.push(GameEvent::EntityStep { entity: ent.handle });
	return true;
}

/// To flick a block is to push it off of a tile that Chip cannot enter.
fn flick(s: &mut GameState, &new_pos: &Vec2i, step_dir: Compass) {
	for ehandle in s.qt.get(new_pos) {
		let Some(mut ent) = s.ents.take(ehandle) else { continue };

		if matches!(ent.kind, EntityKind::Block) {
			if try_move(s, &mut ent, step_dir) {
				s.update_hidden_flag(ent.pos);
				s.update_hidden_flag(ent.pos - step_dir.to_vec());
				s.events.push(GameEvent::BlockPush { entity: ent.handle });
				s.events.push(GameEvent::SoundFx { sound: SoundFx::BlockMoving });
			}
		}

		s.ents.put(ent);
	}
}

pub fn try_terrain_move(s: &mut GameState, ent: &mut Entity, step_dir: Option<Compass>) -> bool {
	let terrain = s.field.get_terrain(ent.pos);
	match terrain {
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
		Terrain::ForceRandom => { let dir = s.rand.compass(); try_move(s, ent, dir) },
		Terrain::Teleport => if let Some(step_dir) = step_dir { teleport(s, ent, step_dir) } else { false },
		_ => return false,
	};
	ent.flags |= EF_MOMENTUM;
	return true;
}

/// Teleports the entity to the destination of a teleporter.
pub fn teleport(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	let old_pos = ent.pos;
	loop {
		// Find the teleport connection
		let Some(conn) = s.field.find_conn_by_src(ent.pos) else { return false };
		// Teleport the entity
		s.qt.update(ent.handle, ent.pos, conn.dest);
		ent.pos = conn.dest;
		// Force the entity to move out of the teleporter
		if try_move(s, ent, step_dir) {
			break;
		}
		// Do nothing if no valid teleport exit is found
		if old_pos == ent.pos {
			return false;
		}
	}

	// Teleport the entity
	s.events.push(GameEvent::EntityTeleport { entity: ent.handle });
	if matches!(ent.kind, EntityKind::Player) {
		s.events.push(GameEvent::SoundFx { sound: SoundFx::Teleporting });
	}
	return true;
}

pub fn interact_terrain(s: &mut GameState, ent: &mut Entity) {
	// Play sound only for player and blocks to avoid a cacophony
	let play_sound = matches!(ent.kind, EntityKind::Player | EntityKind::Block);

	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::BearTrap) {
		let trapped = matches!(s.get_trap_state(ent.pos), TrapState::Closed);
		if trapped && ent.flags & EF_TRAPPED == 0 {
			s.events.push(GameEvent::EntityTrapped { entity: ent.handle });
			// Avoid audio spam when the level is initially loaded
			if s.time != 0 {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::TrapEntered });
			}
		}
		ent.flags = if trapped { ent.flags | EF_TRAPPED } else { ent.flags & !EF_TRAPPED };
	}

	if matches!(ent.kind, EntityKind::Player) {
		if let Some(step_dir) = ent.step_dir {
			let from_pos = ent.pos - step_dir.to_vec();
			if matches!(s.field.get_terrain(from_pos), Terrain::RecessedWall) {
				s.set_terrain(from_pos, Terrain::Wall);
				s.events.push(GameEvent::SoundFx { sound: SoundFx::WallPopup });
			}
		}
	}

	#[inline]
	fn press_once(ent: &mut Entity) -> bool {
		// Checking for EF_NEW_POS prevents entities from pressing buttons when spawned...
		let state = ent.flags & EF_BUTTON_DOWN == 0 && ent.flags & EF_NEW_POS != 0;
		ent.flags |= EF_BUTTON_DOWN;
		state
	}

	match terrain {
		Terrain::GreenButton => {
			if press_once(ent) {
				for y in 0..s.field.height {
					for x in 0..s.field.width {
						let terrain = s.field.get_terrain(Vec2i::new(x, y));
						let new = match terrain {
							Terrain::ToggleFloor => Terrain::ToggleWall,
							Terrain::ToggleWall => Terrain::ToggleFloor,
							_ => continue,
						};
						s.set_terrain(Vec2i::new(x, y), new);
					}
				}
				if play_sound {
					s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
				}
			}
		}
		Terrain::RedButton => {
			if press_once(ent) {
				let Some(conn) = s.field.find_conn_by_src(ent.pos) else { return };

				// Handle CloneBlock tiles separately
				let clone_block_dir = match s.field.get_terrain(conn.dest) {
					Terrain::CloneBlockN => Some(Compass::Up),
					Terrain::CloneBlockW => Some(Compass::Left),
					Terrain::CloneBlockS => Some(Compass::Down),
					Terrain::CloneBlockE => Some(Compass::Right),
					_ => None,
				};

				// Spawn a new entity
				let args = if let Some(clone_block_dir) = clone_block_dir {
					EntityArgs {
						kind: EntityKind::Block,
						pos: conn.dest,
						face_dir: Some(clone_block_dir),
					}
				}
				else {
					// Find the template entity connected to the red button
					let template = s.qt.get(conn.dest)[0];
					let Some(template_ent) = s.ents.get(template) else { return };
					if template_ent.flags & EF_TEMPLATE == 0 {
						return;
					}
					template_ent.to_entity_args()
				};
				s.spawns.push(args);

				if play_sound {
					s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
				}
			}
		}
		Terrain::BrownButton => {
			if press_once(ent) && play_sound {
				s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
			}
		}
		Terrain::BlueButton => {
			if press_once(ent) {
				for other in s.ents.iter_mut() {
					if matches!(other.kind, EntityKind::Tank) {
						if let Some(face_dir) = other.face_dir {
							other.face_dir = Some(face_dir.turn_around());
							s.events.push(GameEvent::EntityTurn { entity: other.handle });
						}
					}
				}
				// Handle the Tank which triggered the button separately
				// as it has been taken out of the entity list
				if matches!(ent.kind, EntityKind::Tank) {
					if let Some(face_dir) = ent.face_dir {
						ent.face_dir = Some(face_dir.turn_around());
						s.events.push(GameEvent::EntityTurn { entity: ent.handle });
					}
				}
				if play_sound {
					s.events.push(GameEvent::SoundFx { sound: SoundFx::ButtonPressed });
				}
			}
		}
		_ => {
			ent.flags &= !EF_BUTTON_DOWN;
		}
	}
}
