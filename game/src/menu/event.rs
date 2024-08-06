
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuEvent {
	CursorMove,

	// Main Events
	NewGame,
	Continue,
	GoToLevel,
	HighScores,
	Options,
	About,
	Exit,
	BackToMainMenu,

	// Pause Events
	Resume,
	Restart,
	BackToPauseMenu,

	// Options Events
	BackgroundMusicOn,
	BackgroundMusicOff,
	SoundEffectsOn,
	SoundEffectsOff,
	DevModeOn,
	DevModeOff,

	// Level select Events
	UnlockLevel,
	SelectLevel { level_index: i32 },

	// Unlock level Events
	EnterPassword { code: [u8; 4] },
	BackToLevelSelect,

	// Game Over Events
	NextLevel,
	Retry,
}
