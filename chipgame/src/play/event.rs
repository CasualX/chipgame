use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: chipcore::SoundFx },
	PlayMusic { music: Option<data::MusicId> },
	PlayLevel,
	Quit,
}
