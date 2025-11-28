use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &DATA,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: BASE_SPD,
		face_dir: args.face_dir,
		step_dir: None,
		step_spd: BASE_SPD,
		step_time: -BASE_SPD,
		flags: 0,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn movement_phase(s: &mut GameState, ent: &mut Entity) {
	if ent.flags & (EF_HIDDEN | EF_TEMPLATE) != 0 {
		return;
	}

	if s.time >= ent.step_time + ent.step_spd {
		try_terrain_move(s, ent, ent.step_dir);
	}
}

fn action_phase(s: &mut GameState, ent: &mut Entity) {
	if ent.flags & (EF_HIDDEN | EF_TEMPLATE) != 0 {
		return;
	}

	ps_attack_pos(s, ent.pos, GameOverReason::Collided);
}

fn terrain_phase(s: &mut GameState, ent: &mut Entity, state: &mut InteractTerrainState) {
	let terrain = s.field.get_terrain(ent.pos);

	if matches!(terrain, Terrain::BearTrap) {
		return bear_trap(s, ent, state);
	}

	if s.time == ent.step_time && ent.flags & EF_NEW_POS != 0 {
		match terrain {
			Terrain::Water => {
				s.set_terrain(ent.pos, Terrain::Ice);
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
				s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
				ent.flags |= EF_REMOVE;
			}
			Terrain::Fire => {
				s.set_terrain(ent.pos, Terrain::Water);
				s.events.fire(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
				s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
				ent.flags |= EF_REMOVE;
			}
			Terrain::Dirt => {
				s.set_terrain(ent.pos, Terrain::Floor);
			}
			Terrain::GreenButton => green_button(s, ent, state),
			Terrain::RedButton => red_button(s, ent, state),
			Terrain::BrownButton => brown_button(s, ent, state),
			Terrain::BlueButton => blue_button(s, ent, state),
			_ => {}
		}
	}
}

static DATA: EntityData = EntityData {
	movement_phase,
	action_phase,
	terrain_phase,
	flags: SolidFlags {
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
	},
};
