use super::*;

/// Time after which Chip returns to idle animation
const IDLE_TIME: Time = 20;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	// There can only be one player
	for ehandle in s.ents.handles() {
		if let Some(ent) = s.ents.get(ehandle) {
			if matches!(ent.kind, EntityKind::Player) {
				s.entity_remove(ent.handle);
			}
		}
	}

	let handle = s.ents.alloc();
	s.ps.ehandle = handle;
	s.ents.put(Entity {
		data: &FUNCS,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: BASE_SPD,
		face_dir: None,
		step_dir: None,
		step_spd: BASE_SPD,
		step_time: -BASE_SPD,
		flags: 0,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn think(s: &mut GameState, ent: &mut Entity) {
	let terrain = s.field.get_terrain(ent.pos);

	// Freeze player if game over
	if s.ps.activity.is_game_over() {
		return;
	}

	// Clear movement after a delay
	if s.time >= ent.step_time + IDLE_TIME {
		if ent.face_dir.is_some() {
			s.events.fire(GameEvent::EntityTurn { entity: ent.handle });
		}
		ent.face_dir = None;
	}
	if s.time >= ent.step_time + ent.step_spd && ent.flags & EF_NEW_POS != 0 {
		if matches!(terrain, Terrain::Fire) && !s.ps.fire_boots {
			ps_activity(s, PlayerActivity::Burned);
			return;
		}
		if matches!(terrain, Terrain::Water) && !s.ps.flippers {
			ps_activity(s, PlayerActivity::Drowned);
			// s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
			return;
		}
		if matches!(terrain, Terrain::Exit) {
			s.events.fire(GameEvent::EntityTurn { entity: ent.handle });
			s.events.fire(GameEvent::Fireworks { pos: ent.pos });
			ps_activity(s, PlayerActivity::Win);
			return;
		}
	}

	// Turn dirt to floor after stepping on it
	if ent.flags & EF_NEW_POS != 0 && matches!(terrain, Terrain::Dirt) {
		s.set_terrain(ent.pos, Terrain::Floor);
		s.events.fire(GameEvent::SoundFx { sound: SoundFx::TileEmptied });
	}

	let activity = match terrain {
		Terrain::Water => {
			if !matches!(s.ps.activity, PlayerActivity::Swimming) {
				s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
			}
			PlayerActivity::Swimming
		},
		Terrain::Ice | Terrain::IceNE | Terrain::IceNW | Terrain::IceSE | Terrain::IceSW => if s.ps.ice_skates { PlayerActivity::Skating } else { PlayerActivity::Sliding },
		Terrain::ForceN | Terrain::ForceW | Terrain::ForceS | Terrain::ForceE | Terrain::ForceRandom => if s.ps.suction_boots { PlayerActivity::Suction } else { PlayerActivity::Sliding },
		_ => PlayerActivity::Walking,
	};
	ps_activity(s, activity);

	// Wait until movement is cleared before accepting new input
	if s.time >= ent.step_time + ent.step_spd {
		let input_dir = s.ps.inbuf.read_dir();

		if s.ps.dev_wtw {
			if let Some(input_dir) = input_dir {
				try_move(s, ent, input_dir);
				return;
			}
		}

		'end_move: {
			if let Some(step_dir) = ent.step_dir {
				if matches!(terrain, Terrain::Teleport) {
					if teleport(s, ent, step_dir) {
						break 'end_move;
					}
				}

				// Handle ice physics
				if !s.ps.ice_skates && matches!(terrain, Terrain::Ice | Terrain::IceNW | Terrain::IceNE | Terrain::IceSW | Terrain::IceSE) {
					let (ice_dir, back_dir) = match step_dir {
						Compass::Up => match terrain {
							Terrain::IceNW => (Compass::Right, Compass::Down),
							Terrain::IceNE => (Compass::Left, Compass::Down),
							Terrain::IceSE => (Compass::Up, Compass::Left),
							Terrain::IceSW => (Compass::Up, Compass::Right),
							_ => (step_dir, step_dir.turn_around()),
						},
						Compass::Left => match terrain {
							Terrain::IceNW => (Compass::Down, Compass::Right),
							Terrain::IceNE => (Compass::Left, Compass::Down),
							Terrain::IceSE => (Compass::Left, Compass::Up),
							Terrain::IceSW => (Compass::Up, Compass::Right),
							_ => (step_dir, step_dir.turn_around()),
						},
						Compass::Down => match terrain {
							Terrain::IceNW => (Compass::Down, Compass::Right),
							Terrain::IceNE => (Compass::Down, Compass::Left),
							Terrain::IceSE => (Compass::Left, Compass::Up),
							Terrain::IceSW => (Compass::Right, Compass::Up),
							_ => (step_dir, step_dir.turn_around()),
						},
						Compass::Right => match terrain {
							Terrain::IceNW => (Compass::Right, Compass::Down),
							Terrain::IceNE => (Compass::Down, Compass::Left),
							Terrain::IceSE => (Compass::Up, Compass::Left),
							Terrain::IceSW => (Compass::Right, Compass::Up),
							_ => (step_dir, step_dir.turn_around()),
						},
					};
					// If the player is blocked, try to turn around
					if !try_move(s, ent, ice_dir) {
						if !try_move(s, ent, back_dir) {
							// Softlocked!
						}
						else {
							s.events.fire(GameEvent::PlayerBump { player: () });
							s.events.fire(GameEvent::SoundFx { sound: SoundFx::CantMove });
						}
					}
					break 'end_move;
				}
			}

			if ent.flags & EF_TRAPPED != 0 {
				break 'end_move;
			}

			// Handle force terrain
			let force_dir = match terrain {
				_ if s.ps.suction_boots => None,
				Terrain::ForceW => Some(Compass::Left),
				Terrain::ForceE => Some(Compass::Right),
				Terrain::ForceN => Some(Compass::Up),
				Terrain::ForceS => Some(Compass::Down),
				Terrain::ForceRandom => Some(s.rand.next()),
				_ => None,
			};
			if let Some(force_dir) = force_dir {
				let override_dir = match force_dir {
					_ if !s.ps.forced_move || ent.flags & EF_TRAPPED != 0 => None,
					Compass::Left | Compass::Right => if input_dir == Some(Compass::Up) { Some(Compass::Up) } else if input_dir == Some(Compass::Down) { Some(Compass::Down) } else { None },
					Compass::Up | Compass::Down => if input_dir == Some(Compass::Left) { Some(Compass::Left) } else if input_dir == Some(Compass::Right) { Some(Compass::Right) } else { None },
				};

				// Consider this a forced move if the player did not step in the direction of the force terrain
				if let Some(override_dir) = override_dir {
					if try_move(s, ent, override_dir) {
						s.ps.forced_move = false;
					}
					else {
						try_move(s, ent, force_dir);
						bump(s, ent, override_dir);
						s.ps.forced_move = true;
					}
				}
				else {
					try_move(s, ent, force_dir);
					s.ps.forced_move = true;
				}

				break 'end_move;
			}
			else {
				s.ps.forced_move = false;
			}

			// Handle player input
			if ent.flags & EF_TRAPPED != 0 { }
			else if let Some(dir) = input_dir {
				if !try_move(s, ent, dir) {
					bump(s, ent, dir);
				}
			}
		}
		s.ps.last_step_dir = ent.step_dir;
	}
}

fn bump(s: &mut GameState, ent: &mut Entity, dir: Compass) {
	// ent.step_spd = ent.base_spd;
	ent.step_time = s.time;
	ent.face_dir = Some(dir);
	s.ps.bonks += 1;
	s.events.fire(GameEvent::PlayerBump { player: () });
	s.events.fire(GameEvent::SoundFx { sound: SoundFx::CantMove });
	s.events.fire(GameEvent::EntityTurn { entity: ent.handle });
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: false,
	water: false,
	exit: false,
	blue_fake: false,
	recessed_wall: false,
	keys: false,
	solid_key: false,
	boots: false,
	chips: false,
	creatures: false,
	player: false,
	thief: false,
	hint: false,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
