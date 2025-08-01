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
			s.events.fire(GameEvent::EntityDrown { entity: ent.handle });
			s.events.fire(GameEvent::WaterSplash { pos: ent.pos });
			ent.flags |= EF_REMOVE;
			return;
		}
		if matches!(terrain, Terrain::Fire) {
			s.events.fire(GameEvent::EntityBurn { entity: ent.handle });
			ent.flags |= EF_REMOVE;
			return;
		}
	}

	if s.time >= ent.step_time + ent.step_spd {
		if try_terrain_move(s, ent, ent.step_dir) { }
		else if let Some(face_dir) = ent.face_dir {
			if try_move(s, ent, face_dir) { }
			else {
				// // Choose a random direction to turn
				// let step_dir = s.rand.compass();
				// if try_move(s, ent, step_dir) { }
				// // Idle if there the chosen direction is blocked
				// else { }

				// Choose a direction to turn
				let step_dirs = [face_dir.turn_left(), face_dir.turn_right()];
				let choice = s.rand.coin_flip();

				if try_move(s, ent, step_dirs[choice as usize]) { }
				else if try_move(s, ent, step_dirs[(!choice) as usize]) { }
				// Only turn around if the other directions are blocked
				else if try_move(s, ent, face_dir.turn_around()) { }
				// Softlocked! Wait until freed
				else { }
			}
		}
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Eaten);
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
	keys: false,
	solid_key: true,
	boots: true,
	chips: true,
	creatures: true,
	player: false,
	thief: true,
	hint: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
