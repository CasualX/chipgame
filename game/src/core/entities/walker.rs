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
		trapped: false,
		hidden: false,
		has_moved: false,
		remove: false,
	});
	s.qt.add(handle, args.pos);
	return handle;
}

fn think(s: &mut GameState, ent: &mut Entity) {
	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::CloneMachine) {
		return;
	}
	if matches!(terrain, Terrain::Water) {
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		ent.remove = true;
		return;
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Eaten);
	}

	if ent.step_dir.is_some() && s.time >= ent.step_time + ent.step_spd {
		ent.step_dir = None;
	}

	if ent.trapped || ent.hidden {
		return;
	}
	if s.time >= ent.step_time + ent.step_spd {
		if let Some(face_dir) = ent.face_dir {
			if try_move(s, ent, face_dir) { }
			else {
				// // Choose a random direction to turn
				// let step_dir = s.rand.compass();
				// if try_move(s, ent, step_dir) { }
				// // Idle if there the chosen direction is blocked
				// else { }

				// Choose a direction to turn
				let step_dirs = [face_dir.turn_left(), face_dir.turn_right()];
				let choice = s.rand.rng.coin_flip();

				if try_move(s, ent, step_dirs[choice as usize]) { }
				else if try_move(s, ent, step_dirs[(!choice) as usize]) { }
				// Only turn around if the other directions are blocked
				else if try_move(s, ent, face_dir.turn_around()) { }
				// Softlocked! Wait until freed
				else { }
			}
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
	pickup: true,
	creature: true,
	player: false,
	thief: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
