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
			s.set_terrain(ent.pos, Terrain::Dirt);
			s.events.fire(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
			s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
			ent.flags |= EF_REMOVE;
		}
	}

	if s.time >= ent.step_time + ent.step_spd {
		try_terrain_move(s, ent, ent.step_dir);
	}

	ps_attack_pos(s, ent.pos, GameOverReason::Collided);
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: true,
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
