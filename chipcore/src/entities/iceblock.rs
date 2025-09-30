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

	if ent.flags & EF_NEW_POS != 0 {
		let terrain = s.field.get_terrain(ent.pos);
		if matches!(terrain, Terrain::Water) {
			s.set_terrain(ent.pos, Terrain::Ice);
			s.events.fire(GameEvent::EntityDrown { entity: ent.handle });
			s.events.fire(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
			s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
			ent.flags |= EF_REMOVE;
		}
		else if matches!(terrain, Terrain::Fire) {
			s.set_terrain(ent.pos, Terrain::Water);
			s.events.fire(GameEvent::EntityBurn { entity: ent.handle });
			s.events.fire(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
			s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
			ent.flags |= EF_REMOVE;
		}
		else if matches!(terrain, Terrain::Dirt) {
			s.set_terrain(ent.pos, Terrain::Floor);
		}
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
	if ent.flags & EF_MOMENTUM != 0 && matches!(terrain, Terrain::BearTrap) {
		if let Some(step_dir) = ent.step_dir {
			return try_move(s, ent, step_dir);
		}
	}
	return false;
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: false,
	water: false,
	exit: true,
	blue_fake: true,
	recessed_wall: true,
	keys: false,
	solid_key: false,
	boots: true,
	chips: true,
	creatures: true,
	player: false,
	thief: true,
	hint: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
