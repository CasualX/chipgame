
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FxEvent {
	Sound(chipty::SoundFx),
	Pause,
	Scout,
	Unpause,
	LevelComplete,
	GameOver,
}
