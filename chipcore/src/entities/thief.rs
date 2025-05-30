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

fn think(s: &mut GameState, ent: &mut Entity) {
	if let Some(pl) = s.ents.get(s.ps.ehandle) {
		if pl.pos == ent.pos && (s.ps.flippers || s.ps.fire_boots || s.ps.ice_skates || s.ps.suction_boots) {
			s.ps.flippers = false;
			s.ps.fire_boots = false;
			s.ps.ice_skates = false;
			s.ps.suction_boots = false;
			s.events.fire(GameEvent::ItemsThief { player: () });
			s.events.fire(GameEvent::SoundFx { sound: SoundFx::BootsStolen });
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
	recessed_wall: true,
	keys: true,
	solid_key: true,
	boots: true,
	chips: true,
	creatures: true,
	player: false,
	thief: true,
	hint: false,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
