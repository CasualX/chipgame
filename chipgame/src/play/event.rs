use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: core::SoundFx },
	PlayMusic { music: Option<data::MusicId> },
	PlayLevel,
	Quit,
}
