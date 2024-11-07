use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &FUNCS,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: BASE_SPD * 2,
		face_dir: args.face_dir,
		step_dir: None,
		step_spd: BASE_SPD * 2,
		step_time: 0,
		flags: 0,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn think(s: &mut GameState, ent: &mut Entity) {
	if ent.flags & (EF_REMOVE | EF_HIDDEN | EF_TEMPLATE) != 0 {
		return;
	}

	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::Water) {
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		ent.flags |= EF_REMOVE;
		return;
	}
	if matches!(terrain, Terrain::Fire) {
		s.events.push(GameEvent::EntityBurn { entity: ent.handle });
		ent.flags |= EF_REMOVE;
		return;
	}

	if s.time >= ent.step_time + ent.step_spd {
		if try_terrain_move(s, ent, ent.step_dir) { }
		else if let Some((first_dir, second_dir)) = chase_dirs(s, ent) {
			if try_move(s, ent, first_dir) { }
			else if try_move(s, ent, second_dir) { }
			// If no legal move, stay put and face in the first direction
			else {
				if ent.face_dir != Some(first_dir) {
					s.events.push(GameEvent::EntityTurn { entity: ent.handle });
				}
				ent.face_dir = Some(first_dir);
			}
		}
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Eaten);
	}
}

fn chase_dirs(s: &GameState, ent: &Entity) -> Option<(Compass, Compass)> {
	let pl = s.ents.get(s.ps.ehandle)?;
	let Vec2i { x, y } = pl.pos - ent.pos;

	// Teeth moves either vertically or horizontally toward Chip one square at a time, always taking the longer path, and vertically if tied.
	// However, if this move would be illegal because of some obstacle, it will go the other way if that is a legal move, and if not,
	// it will stay put until Chip moves somewhere that allows it to make another move.

	if x == 0 && y == 0 {
		None
	}
	else if x.abs() <= -y {
		if x > 0 {
			Some((Compass::Up, Compass::Right))
		}
		else if x < 0 {
			Some((Compass::Up, Compass::Left))
		}
		else {
			Some((Compass::Up, Compass::Up))
		}
	}
	else if x.abs() <= y {
		if x > 0 {
			Some((Compass::Down, Compass::Right))
		}
		else if x < 0 {
			Some((Compass::Down, Compass::Left))
		}
		else {
			Some((Compass::Down, Compass::Down))
		}
	}
	else if x < 0 {
		if y > 0 {
			Some((Compass::Left, Compass::Down))
		}
		else if y < 0 {
			Some((Compass::Left, Compass::Up))
		}
		else {
			Some((Compass::Left, Compass::Left))
		}
	}
	else if x > 0 {
		if y > 0 {
			Some((Compass::Right, Compass::Down))
		}
		else if y < 0 {
			Some((Compass::Right, Compass::Up))
		}
		else {
			Some((Compass::Right, Compass::Right))
		}
	}
	else {
		None
	}
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: true,
	fire: true,
	dirt: true,
	water: false,
	exit: true,
	blue_fake: true,
	recessed_wall: true,
	keys: false,
	boots: true,
	chips: true,
	creatures: true,
	player: false,
	thief: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
