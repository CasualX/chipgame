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
		if try_terrain_move(s, ent, ent.face_dir) { }
		else if let Some(face_dir) = ent.face_dir {
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

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Eaten);
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
