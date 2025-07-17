use super::*;

/// Pickup items.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItemPickup {
	Chip,
	Flippers,
	FireBoots,
	IceSkates,
	SuctionBoots,
	BlueKey,
	RedKey,
	GreenKey,
	YellowKey,
}

/// Key colors.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum KeyColor {
	Blue,
	Red,
	Green,
	Yellow,
}

/// Game events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameEvent {
	EntityCreated { entity: EntityHandle, kind: EntityKind },
	EntityRemoved { entity: EntityHandle, kind: EntityKind },
	EntityStep { entity: EntityHandle },
	EntityTurn { entity: EntityHandle },
	EntityTeleport { entity: EntityHandle },
	EntityHidden { entity: EntityHandle, hidden: bool },
	EntityDrown { entity: EntityHandle },
	EntityBurn { entity: EntityHandle },
	EntityTrapped { entity: EntityHandle },
	PlayerActivity { player: PlayerIndex },
	PlayerBump { player: PlayerIndex },
	BlockPush { entity: EntityHandle },
	ItemPickup { entity: EntityHandle, item: ItemPickup },
	BombExplode { pos: Vec2i },
	WaterSplash { pos: Vec2i },
	Fireworks { pos: Vec2i },
	SocketFilled { pos: Vec2i },
	ItemsThief { player: PlayerIndex },
	LockOpened { pos: Vec2i, key: KeyColor },
	TerrainUpdated { pos: Vec2i, old: Terrain, new: Terrain },
	GameWin { player: PlayerIndex },
	GameOver { player: PlayerIndex },
	SoundFx { sound: SoundFx },
}

#[derive(Default)]
pub struct Events {
	events: Vec<GameEvent>,
}

impl Events {
	#[inline]
	pub fn fire(&mut self, event: GameEvent) {
		unsafe {
			self._fire().write(event);
		}
	}

	#[inline(never)]
	unsafe fn _fire(&mut self) -> *mut GameEvent {
		self.events.reserve(1);
		let index = self.events.len();
		self.events.set_len(index + 1);
		self.events.get_unchecked_mut(index)
	}

	#[inline]
	pub fn take(&mut self) -> Vec<GameEvent> {
		std::mem::take(&mut self.events)
	}
}
