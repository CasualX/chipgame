use super::*;

/// Time after which Chip returns to idle animation
const IDLE_TIME: Time = 20;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ps.ehandle = handle;
	s.ents.put(Entity {
		data: &FUNCS,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: BASE_SPD,
		face_dir: args.face_dir,
		step_dir: None,
		step_spd: BASE_SPD,
		step_time: -BASE_SPD,
		trapped: false,
		hidden: false,
		has_moved: false,
		remove: false,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn think(s: &mut GameState, ent: &mut Entity) {
	let terrain = s.field.get_terrain(ent.pos);
	let orig_dir = ent.step_dir;

	// Freeze player if game over
	if s.ps.activity.is_game_over() {
		return;
	}

	// Clear movement after a delay
	if s.time >= ent.step_time + IDLE_TIME {
		if ent.face_dir.is_some() {
			s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
		}
		ent.face_dir = None;
	}
	if s.time >= ent.step_time + ent.step_spd {
		if ent.step_dir.is_some() {
			if matches!(terrain, Terrain::Fire) && !s.ps.fire_boots {
				ps_activity(s, PlayerActivity::Burned);
				return;
			}
			if matches!(terrain, Terrain::Water) && !s.ps.flippers {
				ps_activity(s, PlayerActivity::Drowned);
				return;
			}
		}

		ent.step_dir = None;
	}

	let activity = match terrain {
		Terrain::Water => PlayerActivity::Swimming,
		Terrain::Ice | Terrain::IceNE | Terrain::IceNW | Terrain::IceSE | Terrain::IceSW => if s.ps.ice_skates { PlayerActivity::Skating } else { PlayerActivity::Sliding },
		Terrain::ForceN | Terrain::ForceW | Terrain::ForceS | Terrain::ForceE | Terrain::ForceRandom => if s.ps.suction_boots { PlayerActivity::Suction } else { PlayerActivity::Sliding },
		_ => PlayerActivity::Walking,
	};
	ps_activity(s, activity);

	// Turn dirt to floor after stepping on it
	if matches!(terrain, Terrain::Dirt) {
		s.field.set_terrain(ent.pos, Terrain::Floor);
	}

	// Wait until movement is cleared before accepting new input
	if s.time >= ent.step_time + ent.step_spd {
		let input_dir = s.ps.inbuf.read_dir();

		// Win condition
		if matches!(terrain, Terrain::Exit) && orig_dir.is_some() {
			s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
			ps_activity(s, PlayerActivity::Win);
			return;
		}

		if s.ps.dev_wtw {
			if let Some(input_dir) = input_dir {
				try_move(s, ent, input_dir);
				return;
			}
		}

		'end_move: {
			// First tick after stepping on a new tile
			if let Some(orig_dir) = orig_dir {
				if matches!(terrain, Terrain::Teleport) {
					teleport(s, ent, orig_dir);
					try_move(s, ent, orig_dir);
					break 'end_move;
				}
				if matches!(terrain, Terrain::Hint) {
					s.events.push(GameEvent::PlayerHint { player: ent.handle, pos: ent.pos });
				}

				// Handle ice physics
				if !s.ps.ice_skates && matches!(terrain, Terrain::Ice | Terrain::IceNW | Terrain::IceNE | Terrain::IceSW | Terrain::IceSE) {
					let (ice_dir, back_dir) = match orig_dir {
						Compass::Up => match terrain {
							Terrain::IceNW => (Compass::Right, Compass::Down),
							Terrain::IceNE => (Compass::Left, Compass::Down),
							Terrain::IceSE => (Compass::Up, Compass::Left),
							Terrain::IceSW => (Compass::Up, Compass::Right),
							_ => (orig_dir, orig_dir.turn_around()),
						},
						Compass::Left => match terrain {
							Terrain::IceNW => (Compass::Down, Compass::Right),
							Terrain::IceNE => (Compass::Left, Compass::Down),
							Terrain::IceSE => (Compass::Left, Compass::Up),
							Terrain::IceSW => (Compass::Up, Compass::Right),
							_ => (orig_dir, orig_dir.turn_around()),
						},
						Compass::Down => match terrain {
							Terrain::IceNW => (Compass::Down, Compass::Right),
							Terrain::IceNE => (Compass::Down, Compass::Left),
							Terrain::IceSE => (Compass::Left, Compass::Up),
							Terrain::IceSW => (Compass::Right, Compass::Up),
							_ => (orig_dir, orig_dir.turn_around()),
						},
						Compass::Right => match terrain {
							Terrain::IceNW => (Compass::Right, Compass::Down),
							Terrain::IceNE => (Compass::Down, Compass::Left),
							Terrain::IceSE => (Compass::Up, Compass::Left),
							Terrain::IceSW => (Compass::Right, Compass::Up),
							_ => (orig_dir, orig_dir.turn_around()),
						},
					};
					// If the player is blocked, try to turn around
					if !try_move(s, ent, ice_dir) {
						if !try_move(s, ent, back_dir) {
							// Softlocked!
						}
					}
					break 'end_move;
				}
			}

			if ent.trapped {
				break 'end_move;
			}

			// Handle force terrain
			let force_dir = match terrain {
				_ if s.ps.suction_boots => None,
				Terrain::ForceW => Some(Compass::Left),
				Terrain::ForceE => Some(Compass::Right),
				Terrain::ForceN => Some(Compass::Up),
				Terrain::ForceS => Some(Compass::Down),
				Terrain::ForceRandom => Some(s.rand.compass()),
				_ => None,
			};
			if let Some(force_dir) = force_dir {
				let override_dir = match force_dir {
					_ if !s.ps.forced_move || ent.trapped => None,
					Compass::Left | Compass::Right => if input_dir == Some(Compass::Up) { Some(Compass::Up) } else if input_dir == Some(Compass::Down) { Some(Compass::Down) } else { None },
					Compass::Up | Compass::Down => if input_dir == Some(Compass::Left) { Some(Compass::Left) } else if input_dir == Some(Compass::Right) { Some(Compass::Right) } else { None },
				};

				// Consider this a forced move if the player did not step in the direction of the force terrain
				if let Some(override_dir) = override_dir {
					if try_move(s, ent, override_dir) {
						s.ps.forced_move = false;
					}
					else {
						s.ps.forced_move = try_move(s, ent, force_dir);
						bump(s, ent, override_dir);
					}
				}
				else {
					s.ps.forced_move = try_move(s, ent, force_dir)
				}

				break 'end_move;
			}
			else {
				s.ps.forced_move = false;
			}

			// Handle player input
			if ent.trapped { }
			else if let Some(dir) = input_dir {
				if !try_move(s, ent, dir) {
					bump(s, ent, dir);
				}
			}
		}
	}
}

fn bump(s: &mut GameState, ent: &mut Entity, dir: Compass) {
	ent.step_spd = ent.base_spd;
	ent.face_dir = Some(dir);
	s.ps.steps += 1;
	s.events.push(GameEvent::PlayerBump { player: ent.handle });
	s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
}

fn try_move(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	let new_pos = ent.pos + step_dir.to_vec();

	let new_terrain = s.field.get_terrain(new_pos);
	match new_terrain {
		Terrain::BlueLock => if s.ps.keys[KeyColor::Blue as usize] > 0 {
			s.field.set_terrain(new_pos, Terrain::Floor);
			s.ps.keys[KeyColor::Blue as usize] -= 1;
			s.events.push(GameEvent::LockOpened { pos: new_pos, key: KeyColor::Blue });
		}
		Terrain::RedLock => if s.ps.keys[KeyColor::Red as usize] > 0 {
			s.field.set_terrain(new_pos, Terrain::Floor);
			s.ps.keys[KeyColor::Red as usize] -= 1;
			s.events.push(GameEvent::LockOpened { pos: new_pos, key: KeyColor::Red });
		}
		Terrain::GreenLock => if s.ps.keys[KeyColor::Green as usize] > 0 {
			s.field.set_terrain(new_pos, Terrain::Floor);
			// s.ps.keys[KeyColor::Green as usize] -= 1; // Green keys are infinite
			s.events.push(GameEvent::LockOpened { pos: new_pos, key: KeyColor::Green });
		}
		Terrain::YellowLock => if s.ps.keys[KeyColor::Yellow as usize] > 0 {
			s.field.set_terrain(new_pos, Terrain::Floor);
			s.ps.keys[KeyColor::Yellow as usize] -= 1;
			s.events.push(GameEvent::LockOpened { pos: new_pos, key: KeyColor::Yellow });
		}
		Terrain::BlueWall => {
			s.field.set_terrain(new_pos, Terrain::Wall);
			s.events.push(GameEvent::BlueWallBumped { pos: new_pos });
		}
		Terrain::BlueFake => {
			s.field.set_terrain(new_pos, Terrain::Floor);
			s.events.push(GameEvent::BlueWallCleared { pos: new_pos });
		}
		Terrain::HiddenWall => {
			s.field.set_terrain(new_pos, Terrain::HiddenWallRevealed);
			s.events.push(GameEvent::HiddenWallBumped { pos: new_pos });
		}
		_ => {}
	}

	let mut success = s.ps.dev_wtw || s.field.can_move(ent.pos, step_dir, &ent.data.flags);
	if success {
		for ehandle in s.ents.handles() {
			let Some(mut ent) = s.ents.take(ehandle) else { continue };
			let mut ictx = InteractContext {
				blocking: false,
				push_dir: step_dir,
			};
			if ent.pos == new_pos {
				interact(s, &mut ent, &mut ictx);
			}
			s.ents.put(ent);
			if ictx.blocking {
				success = false;
				// This is tricky. Consider the following:
				// A block is on top of an item pickup (Chip, etc)
				// If we continued and interacted with all entities, the player can interact with the item pickup through the block
				// To prevent that break here BUT the block must be earlier in the entity list than the item pickup
				break;
			}
		}
	}

	let terrain = s.field.get_terrain(ent.pos);
	// Set the player's move speed
	if !s.ps.suction_boots && matches!(terrain, Terrain::ForceW | Terrain::ForceE | Terrain::ForceN | Terrain::ForceS | Terrain::ForceRandom) {
		ent.base_spd = BASE_SPD / 2;
	}
	else if !s.ps.ice_skates && matches!(terrain, Terrain::Ice | Terrain::IceNE | Terrain::IceSE | Terrain::IceNW | Terrain::IceSW) {
		ent.base_spd = BASE_SPD / 2;
	}
	else {
		ent.base_spd = BASE_SPD;
	}

	s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
	ent.face_dir = Some(step_dir);
	ent.step_time = s.time;
	if success {
		let terrain = s.field.get_terrain(ent.pos);
		if matches!(terrain, Terrain::RecessedWall) {
			s.events.push(GameEvent::RecessedWallRaised { pos: ent.pos });
			s.field.set_terrain(ent.pos, Terrain::RaisedWall);
		}

		s.qt.update(ent.handle, ent.pos, new_pos);
		ent.step_dir = Some(step_dir);
		ent.pos = new_pos;
		ent.has_moved = true;
		interact_terrain(s, ent);

		s.ps.steps += 1;
		s.events.push(GameEvent::EntityStep { entity: ent.handle });
	}
	else {
		ent.base_spd = BASE_SPD;
	}
	ent.step_spd = ent.base_spd;

	return success;
}

fn interact(s: &mut GameState, ent: &mut Entity, ictx: &mut InteractContext) {
	match ent.kind {
		EntityKind::Block => {
			if physics::try_move(s, ent, ictx.push_dir) {
				update_hidden_flag(s, ent.pos);
				update_hidden_flag(s, ent.pos - ictx.push_dir.to_vec());
				s.events.push(GameEvent::BlockPush { entity: ent.handle });
			}
			else {
				ictx.blocking = true;
			}
		}
		EntityKind::Socket => {
			if s.ps.chips >= s.field.chips {
				ent.remove = true;
				ictx.blocking = false;
				s.events.push(GameEvent::SocketFilled { pos: ent.pos });
			}
			else {
				ictx.blocking = true;
			}
		}
		_ => {}
	}
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: false,
	water: false,
	exit: false,
	blue_fake: false,
	pickup: false,
	creature: false,
	player: false,
	thief: false,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
