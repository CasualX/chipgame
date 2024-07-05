use super::*;

/// Solid flags.
#[derive(Debug)]
pub struct SolidFlags {
	pub gravel: bool,
	pub fire: bool,
	pub dirt: bool,
	pub water: bool,
	pub exit: bool,
	pub blue_fake: bool,
	pub pickup: bool,
	pub creature: bool,
	pub player: bool,
	pub thief: bool,
}

/// Checks if the entity can move in the given step direction.
///
/// If no direction is given, checks whether the given position is valid to move to.
pub fn can_move(s: &GameState, mut pos: Vec2i, step_dir: Option<Compass>, flags: &SolidFlags) -> bool {
	let mut terrain = s.field.get_terrain(pos);

	if let Some(step_dir) = step_dir {
		// Check for panels on the current terrain
		let solidf = terrain.solid_flags();
		let panel = match step_dir {
			Compass::Up => PANEL_N,
			Compass::Left => PANEL_W,
			Compass::Down => PANEL_S,
			Compass::Right => PANEL_E,
		};
		// If on a solid wall, allow movement out
		if solidf != SOLID_WALL && (solidf & panel) != 0 {
			return false;
		}

		pos += step_dir.to_vec();
		terrain = s.field.get_terrain(pos);

		// Check the solid flags of the next terrain
		let panel = match step_dir {
			Compass::Up => PANEL_S,
			Compass::Left => PANEL_E,
			Compass::Down => PANEL_N,
			Compass::Right => PANEL_W,
		};
		if terrain.solid_flags() & panel != 0 {
			return false;
		}
	}

	// Check if the terrain is solid
	if terrain.solid_flags() == SOLID_WALL {
		return false;
	}

	// Check if allowed to move on certain terrains
	if flags.gravel && matches!(terrain, Terrain::Gravel) {
		return false;
	}
	if flags.fire && matches!(terrain, Terrain::Fire) {
		return false;
	}
	if flags.dirt && matches!(terrain, Terrain::Dirt) {
		return false;
	}
	if flags.exit && matches!(terrain, Terrain::Exit) {
		return false;
	}
	if flags.water && matches!(terrain, Terrain::Water) {
		return false;
	}
	if flags.blue_fake && matches!(terrain, Terrain::BlueFake) {
		return false;
	}

	for ehandle in s.qt.get(pos) {
		let Some(ent) = s.ents.get(ehandle) else { continue };
		let solid = match ent.kind {
			EntityKind::Player => flags.player,
			EntityKind::Chip => flags.pickup,
			EntityKind::Socket => true,
			EntityKind::Block => true,
			EntityKind::Flippers => flags.pickup,
			EntityKind::FireBoots => flags.pickup,
			EntityKind::IceSkates => flags.pickup,
			EntityKind::SuctionBoots => flags.pickup,
			EntityKind::BlueKey => flags.pickup,
			EntityKind::RedKey => flags.pickup,
			EntityKind::GreenKey => flags.pickup,
			EntityKind::YellowKey => flags.pickup,
			EntityKind::Thief => flags.thief,
			EntityKind::Bomb => false,
			EntityKind::Bug => flags.creature,
			EntityKind::FireBall => flags.creature,
			EntityKind::PinkBall => flags.creature,
			EntityKind::Tank => flags.creature,
			EntityKind::Glider => flags.creature,
			EntityKind::Teeth => flags.creature,
			EntityKind::Walker => flags.creature,
			EntityKind::Blob => flags.creature,
			EntityKind::Paramecium => flags.creature,
		};
		if solid {
			return false;
		}
	}

	return true;
}

/// Tries to move the entity in the given step direction.
pub fn try_move(s: &mut GameState, ent: &mut Entity, step_dir: Compass) -> bool {
	if ent.base_spd == 0 || ent.trapped {
		return false;
	}

	if !can_move(s, ent.pos, Some(step_dir), &ent.data.flags) {
		return false;
	}

	// Set the entity's step speed based on the terrain
	let terrain = s.field.get_terrain(ent.pos);
	ent.step_spd = match terrain {
		Terrain::ForceW | Terrain::ForceE | Terrain::ForceN | Terrain::ForceS | Terrain::ForceRandom => cmp::max(1, ent.base_spd / 2),
		Terrain::Ice | Terrain::IceNE | Terrain::IceSE | Terrain::IceNW | Terrain::IceSW => cmp::max(1, ent.base_spd / 2),
		_ => ent.base_spd,
	};

	let new_pos = ent.pos + step_dir.to_vec();
	s.qt.update(ent.handle, ent.pos, new_pos);

	ent.face_dir = Some(step_dir);
	ent.step_dir = Some(step_dir);
	ent.step_time = s.time;
	ent.pos = new_pos;
	ent.has_moved = true;
	s.events.push(GameEvent::EntityFaceDir { entity: ent.handle });
	s.events.push(GameEvent::EntityStep { entity: ent.handle });
	return true;
}

/// Teleports the entity to the destination of a teleporter.
pub fn teleport(s: &mut GameState, ent: &mut Entity, step_dir: Compass) {
	let mut new_pos = ent.pos;
	loop {
		// Find the teleport connection
		let Some(conn) = s.field.find_conn_by_src(new_pos) else { return };
		// Check if the entity can step out of the teleporter
		new_pos = conn.dest;
		if can_move(s, new_pos, Some(step_dir), &ent.data.flags) {
			break;
		}
		// Do nothing if no valid teleport exit is found
		if new_pos == ent.pos {
			return;
		}
	}

	// Teleport the entity
	s.qt.update(ent.handle, ent.pos, new_pos);
	ent.pos = new_pos;
	s.events.push(GameEvent::EntityTeleport { entity: ent.handle });
}
