
/// Sound effects.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SoundFx {
	GameOver,
	GameWin,
	TimeLow,
	Derezz,
	CantMove,
	ICCollected,
	KeyCollected,
	BootCollected,
	BootsStolen,
	Teleporting,
	LockOpened,
	SocketOpened,
	ButtonPressed,
	TileEmptied,
	WallCreated,
	BlueWallCleared,
	TrapEntered,
	BombExplosion,
	WaterSplash,
	OneShotCount,
	BlockMoving,
	SkatingForward,
	SkatingTurn,
	Sliding,
	SlideWalking,
	IceWalking,
	WaterWalking,
	FireWalking,
	WallPopup,
	CursorMove,
	CursorSelect,
}

impl std::str::FromStr for SoundFx {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"GameOver" => Ok(SoundFx::GameOver),
			"GameWin" => Ok(SoundFx::GameWin),
			"TimeLow" => Ok(SoundFx::TimeLow),
			"Derezz" => Ok(SoundFx::Derezz),
			"CantMove" => Ok(SoundFx::CantMove),
			"ICCollected" => Ok(SoundFx::ICCollected),
			"KeyCollected" => Ok(SoundFx::KeyCollected),
			"BootCollected" => Ok(SoundFx::BootCollected),
			"BootsStolen" => Ok(SoundFx::BootsStolen),
			"Teleporting" => Ok(SoundFx::Teleporting),
			"LockOpened" => Ok(SoundFx::LockOpened),
			"SocketOpened" => Ok(SoundFx::SocketOpened),
			"ButtonPressed" => Ok(SoundFx::ButtonPressed),
			"TileEmptied" => Ok(SoundFx::TileEmptied),
			"WallCreated" => Ok(SoundFx::WallCreated),
			"BlueWallCleared" => Ok(SoundFx::BlueWallCleared),
			"TrapEntered" => Ok(SoundFx::TrapEntered),
			"BombExplosion" => Ok(SoundFx::BombExplosion),
			"WaterSplash" => Ok(SoundFx::WaterSplash),
			"OneShotCount" => Ok(SoundFx::OneShotCount),
			"BlockMoving" => Ok(SoundFx::BlockMoving),
			"SkatingForward" => Ok(SoundFx::SkatingForward),
			"SkatingTurn" => Ok(SoundFx::SkatingTurn),
			"Sliding" => Ok(SoundFx::Sliding),
			"SlideWalking" => Ok(SoundFx::SlideWalking),
			"IceWalking" => Ok(SoundFx::IceWalking),
			"WaterWalking" => Ok(SoundFx::WaterWalking),
			"FireWalking" => Ok(SoundFx::FireWalking),
			"WallPopup" => Ok(SoundFx::WallPopup),
			"CursorMove" => Ok(SoundFx::CursorMove),
			"CursorSelect" => Ok(SoundFx::CursorSelect),
			_ => Err(()),
		}
	}
}
