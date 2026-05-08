
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FxEvent {
	PlaySound(chipty::SoundFx),
	PauseGame,
	ScoutMode,
	ResumePlay,
	LevelComplete,
	GameOver,
}
