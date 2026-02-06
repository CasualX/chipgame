use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &DATA,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: 0,
		face_dir: args.face_dir,
		step_dir: None,
		step_spd: 0,
		step_time: 0,
		flags: 0,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn movement_phase(_s: &mut GameState, _phase: &mut MovementPhase, _ent: &mut Entity) {
}

fn action_phase(s: &mut GameState, _phase: &mut ActionPhase, ent: &mut Entity) {
	update_hidden_flag(s, ent);

	if ps_check_new_pos(s, ent.pos) {
		let boots = s.ps.boots;
		if boots != Boots::NONE {
			s.ps.boots = Boots::NONE;
			s.events.fire(GameEvent::ItemsThief { player: (), boots });
			s.events.fire(GameEvent::SoundFx { sound: SoundFx::BootsStolen });
		}
	}
}

fn terrain_phase(_s: &mut GameState, _phase: &mut TerrainPhase, _ent: &mut Entity) {
}

static DATA: EntityData = EntityData {
	movement_phase,
	action_phase,
	terrain_phase,
	flags: SolidFlags {
		gravel: false,
		fire: true,
		dirt: true,
		water: false,
		exit: true,
		blue_fake: true,
		recessed_wall: true,
		keys: true,
		boots: true,
		chips: true,
		creatures: true,
		player: false,
		thief: true,
		hint: false,
	},
	hidden: HiddenData {
		dirt: false,
	},
};
