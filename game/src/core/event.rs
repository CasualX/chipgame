use super::*;

/// Pickup items.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Pickup {
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
	EntityFaceDir { entity: EntityHandle },
	EntityTeleport { entity: EntityHandle },
	EntityHidden { entity: EntityHandle, hidden: bool },
	EntityDrown { entity: EntityHandle },
	PlayerAction { player: EntityHandle },
	PlayerHint { player: EntityHandle, pos: Vec2i },
	ItemPickup { player: EntityHandle, kind: Pickup },
	BombExplode { entity: EntityHandle },
	SocketFilled { pos: Vec2i },
	ItemsThief { player: EntityHandle },
	LockRemoved { pos: Vec2i, key: KeyColor },
	BlueWallBumped { pos: Vec2i },
	BlueWallCleared { pos: Vec2i },
	HiddenWallBumped { pos: Vec2i },
	RecessedWallRaised { pos: Vec2i },
	GreenButton { entity: EntityHandle, pressed: bool },
	RedButton { entity: EntityHandle, pressed: bool },
	BrownButton { entity: EntityHandle, pressed: bool },
	BlueButton { entity: EntityHandle, pressed: bool },
	GameWin { player: EntityHandle },
	GameOver { player: EntityHandle },
}
