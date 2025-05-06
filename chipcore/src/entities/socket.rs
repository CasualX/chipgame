use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &FUNCS,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: 0,
		face_dir: None,
		step_dir: None,
		step_spd: 0,
		step_time: 0,
		flags: 0,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn think(_s: &mut GameState, _ent: &mut Entity) {
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: false,
	water: false,
	exit: false,
	blue_fake: false,
	recessed_wall: false,
	keys: true,
	solid_key: true,
	boots: true,
	chips: true,
	creatures: true,
	player: true,
	thief: false,
	hint: false,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
