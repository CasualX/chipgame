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
		s.field.set_terrain(ent.pos, Terrain::Dirt);
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		s.events.push(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
		ent.flags |= EF_REMOVE;
	}

	if s.time >= ent.step_time + ent.step_spd {
		if try_special_move(s, ent) { }
		else if try_terrain_move(s, ent, ent.step_dir) { }
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Collided);
	}
}

fn try_special_move(s: &mut GameState, ent: &mut Entity) -> bool {
	// Simulate Lynx behavior when an object is released from a trap
	// This is relevant to solve level 109 (Torturechamber)
	// FIXME: Do this for all objects that are released from traps
	let terrain = s.field.get_terrain(ent.pos);
	if ent.flags & EF_FORCED_MOVE != 0 && matches!(terrain, Terrain::BearTrap) {
		if let Some(step_dir) = ent.step_dir {
			return try_move(s, ent, step_dir);
		}
	}
	return false;
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: true,
	water: false,
	exit: true,
	blue_fake: true,
	recessed_wall: false,
	items: false,
	chips: true,
	creatures: true,
	player: false,
	thief: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
