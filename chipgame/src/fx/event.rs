use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FxEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<data::MusicId> },
	Pause,
	Scout,
	Unpause,
	GameWin,
	GameOver,
}
