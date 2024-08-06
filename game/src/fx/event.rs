use super::*;

pub enum FxEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<MusicId> },
	Pause,
	Unpause,
}
