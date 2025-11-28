use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &DATA,
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

fn movement_phase(_s: &mut GameState, _ent: &mut Entity) {
}

fn action_phase(s: &mut GameState, ent: &mut Entity) {
	if ps_check_new_pos(s, ent.pos) {
		if s.ps.flippers || s.ps.fire_boots || s.ps.ice_skates || s.ps.suction_boots {
			s.ps.flippers = false;
			s.ps.fire_boots = false;
			s.ps.ice_skates = false;
			s.ps.suction_boots = false;
			s.events.fire(GameEvent::ItemsThief { player: () });
			s.events.fire(GameEvent::SoundFx { sound: SoundFx::BootsStolen });
		}
	}
}

fn terrain_phase(_s: &mut GameState, _ent: &mut Entity, _state: &mut InteractTerrainState) {
}

static DATA: EntityData = EntityData {
	movement_phase,
	action_phase,
	terrain_phase,
	flags: SolidFlags {
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
	},
};
