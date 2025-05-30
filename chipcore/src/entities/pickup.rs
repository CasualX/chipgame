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
	if let Some(pl) = s.ents.get(s.ps.ehandle) {
		if pl.pos == ent.pos && pl.flags & EF_NEW_POS != 0 {
			pickup_item(s, ent);
		}
	}
}

fn pickup_item(s: &mut GameState, ent: &mut Entity) {
	if ent.flags & (EF_REMOVE | EF_HIDDEN | EF_TEMPLATE) != 0 {
		return;
	}

	ent.flags |= EF_REMOVE;

	let (item, sound) = match ent.kind {
		EntityKind::Chip => {
			s.ps.chips += 1;
			(ItemPickup::Chip, SoundFx::ICCollected)
		},
		EntityKind::BlueKey => {
			s.ps.keys[KeyColor::Blue as usize] += 1;
			(ItemPickup::BlueKey, SoundFx::KeyCollected)
		},
		EntityKind::RedKey => {
			s.ps.keys[KeyColor::Red as usize] += 1;
			(ItemPickup::RedKey, SoundFx::KeyCollected)
		},
		EntityKind::GreenKey => {
			s.ps.keys[KeyColor::Green as usize] += 1;
			(ItemPickup::GreenKey, SoundFx::KeyCollected)
		},
		EntityKind::YellowKey => {
			s.ps.keys[KeyColor::Yellow as usize] += 1;
			(ItemPickup::YellowKey, SoundFx::KeyCollected)
		},
		EntityKind::Flippers => {
			s.ps.flippers = true;
			(ItemPickup::Flippers, SoundFx::BootCollected)
		},
		EntityKind::FireBoots => {
			s.ps.fire_boots = true;
			(ItemPickup::FireBoots, SoundFx::BootCollected)
		},
		EntityKind::IceSkates => {
			s.ps.ice_skates = true;
			(ItemPickup::IceSkates, SoundFx::BootCollected)
		},
		EntityKind::SuctionBoots => {
			s.ps.suction_boots = true;
			(ItemPickup::SuctionBoots, SoundFx::BootCollected)
		},
		_ => return,
	};

	s.events.fire(GameEvent::ItemPickup { entity: ent.handle, item });
	s.events.fire(GameEvent::SoundFx { sound });
}

const FLAGS: SolidFlags = SolidFlags {
	gravel: true,
	fire: true,
	dirt: true,
	water: true,
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
	hint: true,
};

static FUNCS: EntityData = EntityData { think, flags: FLAGS };
