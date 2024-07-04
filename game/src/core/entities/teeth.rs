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
	if matches!(terrain, Terrain::CloneMachine) && ent.step_dir.is_none() {
		return;
	}
	if matches!(terrain, Terrain::Water) {
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		ent.remove = true;
		return;
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Eaten);
	}

	if ent.step_dir.is_some() && s.time >= ent.step_time + ent.step_spd {
		ent.step_dir = None;
	}

	if ent.trapped || ent.hidden {
		return;
	}
	if s.time >= ent.step_time + ent.step_spd {
		if let Some((first_dir, second_dir)) = chase_dirs(s, ent) {
			if try_move(s, ent, first_dir) { }
			else if try_move(s, ent, second_dir) { }
			// If no legal move, stay put and face in the first direction
			else {
				if ent.face_dir != Some(first_dir) {
					s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
				}
				ent.face_dir = Some(first_dir);
			}
		}
	}
}

fn chase_dirs(s: &GameState, ent: &Entity) -> Option<(Compass, Compass)> {
	let pl = s.ents.get(s.ps.ehandle)?;
	let d = pl.pos - ent.pos;

	// Teeth moves either vertically or horizontally toward Chip one square at a time, always taking the longer path, and vertically if tied.
	// However, if this move would be illegal because of some obstacle, it will go the other way if that is a legal move, and if not,
	// it will stay put until Chip moves somewhere that allows it to make another move.

	if d.y == 0 {
		if d.x > 0 {
			Some((Compass::Right, Compass::Right))
		}
		else if d.x < 0 {
			Some((Compass::Left, Compass::Left))
		}
		else {
			None
		}
	}
	else if d.y > 0 {
		if d.x > d.y {
			Some((Compass::Right, Compass::Down))
		}
		else if d.x > 0 {
			Some((Compass::Down, Compass::Right))
		}
		else if d.x == 0 {
			Some((Compass::Down, Compass::Down))
		}
		else if d.x < d.y {
			Some((Compass::Left, Compass::Down))
		}
		else {
			Some((Compass::Down, Compass::Left))
		}
	}
	else/* if d.y < 0*/ {
		if d.x > -d.y {
			Some((Compass::Right, Compass::Up))
		}
		else if d.x > 0 {
			Some((Compass::Up, Compass::Right))
		}
		else if d.x == 0 {
			Some((Compass::Up, Compass::Up))
		}
		else if d.x < -d.y {
			Some((Compass::Left, Compass::Up))
		}
		else {
			Some((Compass::Up, Compass::Left))
		}
	}
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: true,
	fire: true,
	dirt: true,
	water: false,
	exit: true,
	blue_fake: true,
	pickup: true,
	creature: true,
	player: false,
	thief: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
