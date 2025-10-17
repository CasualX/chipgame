
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MenuEvent {
	CursorMove,
	CursorSelect,
	CloseMenu,

	LoadLevelPack { index: usize },
	OpenMainMenu,

	// MainMenu Events
	NewGame,
	Continue,
	OpenGoToLevel,
	OpenOptions,
	OpenAbout,
	ExitGame,

	// GoToLevel Events
	OpenUnlockLevel,
	PreviewLevel { level_number: i32 },
	PreviewExit,
	PlayLevel { level_number: i32 },

	// UnlockLevel Events
	EnterPassword { code: [u8; 4] },

	// OptionsMenu Events
	SetBackgroundMusic { value: bool },
	SetSoundEffects { value: bool },
	SetDeveloperMode { value: bool },
	SetPerspective { value: bool },

	// PauseMenu Events
	OpenPauseMenu,
	RestartLevel,
	PlayNextLevel,
	RetryLevel,
	SaveReplay,
	ResumePlay,

	// ScoutMode Events
	OpenScoutMode,
	ScoutN,
	ScoutE,
	ScoutS,
	ScoutW,
}
