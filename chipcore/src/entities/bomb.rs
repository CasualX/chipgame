use super::*;

pub fn create(s: &mut GameState, args: &EntityArgs) -> EntityHandle {
	let handle = s.ents.alloc();
	s.ents.put(Entity {
		data: &FUNCS,
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

fn think(s: &mut GameState, ent: &mut Entity) {
	if ent.flags & (EF_REMOVE | EF_HIDDEN | EF_TEMPLATE) != 0 {
		return;
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Bombed);
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
		exploded = true;
		if matches!(other_ent.kind, EntityKind::Player) {
			continue;
		}
		other_ent.flags |= EF_REMOVE;
	}

	if exploded {
		ent.flags |= EF_REMOVE;
		s.events.push(GameEvent::BombExplode { entity: ent.handle });
		s.events.push(GameEvent::SoundFx { sound: SoundFx::BombExplosion });
	}
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: false,
	fire: false,
	dirt: false,
	water: false,
	exit: false,
	blue_fake: false,
	recessed_wall: false,
	keys: false,
	boots: true,
	chips: true,
	creatures: false,
	player: false,
	thief: false,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
