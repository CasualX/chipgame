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
	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Collided);
	}

	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::Water) {
		s.field.set_terrain(ent.pos, Terrain::Dirt);
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		ent.remove = true;
	}

	if ent.step_dir.is_some() && s.time >= ent.step_time + ent.step_spd {
		let step_dir = ent.step_dir.unwrap();
		if let Some((ice_dir, back_dir)) = ice_dir(terrain, step_dir) {
			if try_move(s, ent, ice_dir) { }
			else if try_move(s, ent, back_dir) { }
			else {
				ent.step_dir = None;
			}
		}
		else {
			ent.step_dir = None;
		}
	}
}

pub fn ice_dir(terrain: Terrain, dir: Compass) -> Option<(Compass, Compass)> {
	let x = match dir {
		Compass::Up => match terrain {
			Terrain::IceNW => (Compass::Right, Compass::Down),
			Terrain::IceNE => (Compass::Left, Compass::Down),
			Terrain::IceSE => (Compass::Up, Compass::Left),
			Terrain::IceSW => (Compass::Up, Compass::Right),
			Terrain::Ice => (dir, dir.turn_around()),
			_ => return None,
		},
		Compass::Left => match terrain {
			Terrain::IceNW => (Compass::Down, Compass::Right),
			Terrain::IceNE => (Compass::Left, Compass::Down),
			Terrain::IceSE => (Compass::Left, Compass::Up),
			Terrain::IceSW => (Compass::Up, Compass::Right),
			Terrain::Ice => (dir, dir.turn_around()),
			_ => return None,
		},
		Compass::Down => match terrain {
			Terrain::IceNW => (Compass::Down, Compass::Right),
			Terrain::IceNE => (Compass::Down, Compass::Left),
			Terrain::IceSE => (Compass::Left, Compass::Up),
			Terrain::IceSW => (Compass::Right, Compass::Up),
			Terrain::Ice => (dir, dir.turn_around()),
			_ => return None,
		},
		Compass::Right => match terrain {
			Terrain::IceNW => (Compass::Right, Compass::Down),
			Terrain::IceNE => (Compass::Down, Compass::Left),
			Terrain::IceSE => (Compass::Up, Compass::Left),
			Terrain::IceSW => (Compass::Right, Compass::Up),
			Terrain::Ice => (dir, dir.turn_around()),
			_ => return None,
		},
	};
	Some(x)
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: true,
	fire: false,
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
