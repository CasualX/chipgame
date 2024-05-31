use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &FUNCS,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: BASE_SPD,
		face_dir: args.face_dir,
		step_dir: None,
		step_spd: BASE_SPD,
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
	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Death);
	}

	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::CloneMachine) && ent.step_dir.is_none() {
		return;
	}
	if matches!(terrain, Terrain::Water) {
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		ent.remove = true;
		return;
	}

	if s.time >= ent.step_time + ent.step_spd {
		ent.step_dir = None;
	}

	if ent.trapped || ent.hidden {
		return;
	}
	if s.time >= ent.step_time + ent.step_spd {
		if let Some(face_dir) = ent.face_dir {
			// If bug can turn left, turn left
			if try_move(s, ent, face_dir.turn_left()) { }
			// Otherwise try to move forward
			else if try_move(s, ent, face_dir) { }
			// If forward is blocked, try to turn right
			else if try_move(s, ent, face_dir.turn_right()) { }
			// At this point, can't turn left, can't go forward, can't turn right so try to turn around
			else if try_move(s, ent, face_dir.turn_around()) { }
			// Trapped! Wait until freed
			else { }
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
