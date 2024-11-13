
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuEvent {
	CursorMove,
	CloseMenu,

	// Main Events
	NewGame,
	Continue,
	LevelSelect,
	HighScores,
	Options,
	About,
	Exit,
	MainMenu,

	// Pause Events
	Resume,
	Restart,
	PauseMenu,

	// Options Events
	BgMusicOn,
	BgMusicOff,
	SoundFxOn,
	SoundFxOff,
	DevModeOn,
	DevModeOff,

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
