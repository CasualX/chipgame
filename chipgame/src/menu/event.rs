
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuEvent {
	CursorMove,
	CursorSelect,
	CloseMenu,
	LevelPreview { level_number: i32 },

	// Main Events
	LevelPackSelect { index: usize },
	NewGame,
	Continue,
	LevelSelect,
	Options,
	About,
	Exit,
	MainMenu,

	// Pause Events
	Resume,
	Restart,
	PauseMenu,

	// Options Events
	SetBackgroundMusic { value: bool },
	SetSoundEffects { value: bool },
	SetDeveloperMode { value: bool },
	SetPerspective { value: bool },

	// Level select Events
	UnlockLevel,
	PlayLevel { level_number: i32 },

	// Unlock level Events
	EnterPassword { code: [u8; 4] },

	// Game Over Events
	NextLevel,
	Retry,
	SaveReplay,
}
