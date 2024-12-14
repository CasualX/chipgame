use super::*;

const BASE_SPD: Time = 12;

mod blob;
mod block;
mod bomb;
mod bug;
mod fireball;
mod glider;
mod paramecium;
mod pickup;
mod player;
mod pinkball;
mod socket;
mod tank;
mod teeth;
mod thief;
mod walker;

/// Entity construction arguments.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug)]
pub struct EntityArgs {
	pub kind: EntityKind,
	pub pos: Vec2i,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub face_dir: Option<Compass>,
}

impl GameState {
	/// Creates an entity from arguments.
	pub fn entity_create(&mut self, data: &EntityArgs) -> EntityHandle {
		// Don't create entities outside the field
		if data.pos.x < 0 || data.pos.x >= self.field.width || data.pos.y < 0 || data.pos.y >= self.field.height {
			return EntityHandle::INVALID;
		}

		let s = self;
		let ehandle = match data.kind {
			EntityKind::Player => player::create(s, data),
			EntityKind::Chip => pickup::create(s, data),
			EntityKind::Socket => socket::create(s, data),
			EntityKind::Block => block::create(s, data),
			EntityKind::Flippers => pickup::create(s, data),
			EntityKind::FireBoots => pickup::create(s, data),
			EntityKind::IceSkates => pickup::create(s, data),
			EntityKind::SuctionBoots => pickup::create(s, data),
			EntityKind::BlueKey => pickup::create(s, data),
			EntityKind::RedKey => pickup::create(s, data),
			EntityKind::GreenKey => pickup::create(s, data),
			EntityKind::YellowKey => pickup::create(s, data),
			EntityKind::Thief => thief::create(s, data),
			EntityKind::Bomb => bomb::create(s, data),
			EntityKind::Bug => bug::create(s, data),
			EntityKind::FireBall => fireball::create(s, data),
			EntityKind::PinkBall => pinkball::create(s, data),
			EntityKind::Tank => tank::create(s, data),
			EntityKind::Glider => glider::create(s, data),
			EntityKind::Teeth => teeth::create(s, data),
			EntityKind::Walker => walker::create(s, data),
			EntityKind::Blob => blob::create(s, data),
			EntityKind::Paramecium => paramecium::create(s, data),
		};
		s.events.fire(GameEvent::EntityCreated { entity: ehandle, kind: data.kind });
		return ehandle;
	}

	/// Removes an entity from the game.
	pub fn entity_remove(&mut self, ehandle: EntityHandle) -> Option<EntityArgs> {
		let s = self;
		let ent = s.ents.remove(ehandle)?;
		s.qt.remove(ehandle, ent.pos);
		s.events.fire(GameEvent::EntityRemoved { entity: ehandle, kind: ent.kind });
		Some(ent.to_entity_args())
	}
}
