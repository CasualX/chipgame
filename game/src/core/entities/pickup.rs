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
	if let Some(pl) = s.ents.get(s.ps.ehandle) {
		if pl.pos == ent.pos {
			pickup_item(s, ent);
		}
	}
}

fn pickup_item(s: &mut GameState, ent: &mut Entity) {
	if /*ent.hidden || */ent.remove {
		return;
	}

	ent.remove = true;

	let item = match ent.kind {
		EntityKind::Chip => { s.ps.chips += 1; ItemPickup::Chip },
		EntityKind::BlueKey => { s.ps.keys[KeyColor::Blue as usize] += 1; ItemPickup::BlueKey },
		EntityKind::RedKey => { s.ps.keys[KeyColor::Red as usize] += 1; ItemPickup::RedKey },
		EntityKind::GreenKey => { s.ps.keys[KeyColor::Green as usize] += 1; ItemPickup::GreenKey },
		EntityKind::YellowKey => { s.ps.keys[KeyColor::Yellow as usize] += 1; ItemPickup::YellowKey },
		EntityKind::Flippers => { s.ps.flippers = true; ItemPickup::Flippers },
		EntityKind::FireBoots => { s.ps.fire_boots = true; ItemPickup::FireBoots },
		EntityKind::IceSkates => { s.ps.ice_skates = true; ItemPickup::IceSkates },
		EntityKind::SuctionBoots => { s.ps.suction_boots = true; ItemPickup::SuctionBoots },
		_ => return,
	};

	s.events.push(GameEvent::ItemPickup { entity: ent.handle, item });
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: true,
	fire: true,
	dirt: true,
	water: true,
	exit: true,
	blue_fake: true,
	pickup: true,
	creature: true,
	player: false,
	thief: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
