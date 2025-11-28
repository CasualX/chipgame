use super::*;

/// Pickup items.
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
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum KeyColor {
	Blue,
	Red,
	Green,
	Yellow,
}

/// Reasons for game over.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GameOverReason {
	/// Level is completed.
	LevelComplete,
	/// Drowned in water without flippers.
	Drowned,
	/// Stepped in fire without fire boots.
	Burned,
	/// Stepped on a bomb.
	Bombed,
	/// Collided with a block.
	Collided,
	/// Eaten by a monster.
	Eaten,
	/// Ran out of time.
	TimeOut,
	/// The player entity does not exist.
	NotOkay,
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
	EntityTrapped { entity: EntityHandle },
	PlayerGameOver { entity: EntityHandle, reason: GameOverReason },
	PlayerActivity { entity: EntityHandle },
	PlayerBump { entity: EntityHandle },
	BlockPush { entity: EntityHandle },
	ItemPickup { entity: EntityHandle, item: ItemPickup },
	FireHidden { pos: Vec2i, hidden: bool },
	BombExplode { pos: Vec2i },
	WaterSplash { pos: Vec2i },
	SocketFilled { pos: Vec2i },
	ItemsThief { player: PlayerIndex },
	LockOpened { pos: Vec2i, key: KeyColor },
	TerrainUpdated { pos: Vec2i, old: Terrain, new: Terrain },
	GameOver { reason: GameOverReason },
	SoundFx { sound: SoundFx },
}

#[derive(Default)]
pub struct Events {
	events: Vec<GameEvent>,
}

impl Into<Vec<GameEvent>> for Events {
	#[inline]
	fn into(self) -> Vec<GameEvent> {
		self.events
	}
}

impl Events {
	#[inline]
	pub fn clear(&mut self) {
		self.events.clear();
	}

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
		mem::take(&mut self.events)
	}
}
