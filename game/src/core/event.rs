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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
	PlayerActivity { player: EntityHandle },
	PlayerHint { player: EntityHandle, pos: Vec2i },
	PlayerBump { player: EntityHandle },
	BlockPush { entity: EntityHandle },
	ItemPickup { entity: EntityHandle, item: ItemPickup },
	BombExplode { entity: EntityHandle },
	SocketFilled { pos: Vec2i },
	ItemsThief { player: EntityHandle },
	LockOpened { pos: Vec2i, key: KeyColor },
	TerrainUpdated { pos: Vec2i, old: Terrain, new: Terrain },
	GameWin { player: EntityHandle },
	GameOver { player: EntityHandle },
	SoundFx { sound: SoundFx },
}
