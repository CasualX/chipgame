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

	let terrain = s.field.get_terrain(ent.pos);
	if matches!(terrain, Terrain::Water) {
		s.field.set_terrain(ent.pos, Terrain::Dirt);
		s.events.push(GameEvent::EntityDrown { entity: ent.handle });
		s.events.push(GameEvent::SoundFx { sound: SoundFx::WaterSplash });
		ent.flags |= EF_REMOVE;
	}

	if s.time >= ent.step_time + ent.step_spd {
		if let Some(step_dir) = ent.step_dir {
			if try_terrain_move(s, ent, step_dir) { }
		}
	}

	if s.ents.get(s.ps.ehandle).map(|e| e.pos) == Some(ent.pos) {
		ps_activity(s, PlayerActivity::Collided);
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
