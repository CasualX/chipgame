use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &DATA,
		handle,
		kind: args.kind,
		pos: args.pos,
		base_spd: BASE_SPD,
		face_dir: None,
		step_dir: None,
		step_spd: BASE_SPD,
		step_time: 0,
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

	let mut exploded = false;
	for ehandle in s.qt.get(ent.pos) {
		if ehandle == ent.handle {
			continue;
		}
		let Some(other_ent) = s.ents.get_mut(ehandle) else { continue };
		// HACK! Delay expolosion by 1 tick to work around animation bug
		if other_ent.step_time >= s.time {
			return;
		}
		// Only explode when an entity moved into the bomb
		if other_ent.flags & EF_NEW_POS == 0 {
			continue;
		}
		exploded = true;
		// Remove non-player entities
		if matches!(other_ent.kind, EntityKind::Player) {
			ps_attack(s, ehandle, GameOverReason::Bombed);
		}
		else {
			other_ent.flags |= EF_REMOVE;
		}
	}

	if exploded {
		ent.flags |= EF_REMOVE;
		s.events.fire(GameEvent::BombExplode { pos: ent.pos });
		s.events.fire(GameEvent::SoundFx { sound: SoundFx::BombExplosion });
	}
}

fn terrain_phase(_s: &mut GameState, _ent: &mut Entity, _state: &mut InteractTerrainState) {
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
		exit: false,
		blue_fake: false,
		recessed_wall: false,
		keys: false,
		solid_key: true,
		boots: true,
		chips: true,
		creatures: false,
		player: false,
		thief: false,
		hint: false,
	},
};
