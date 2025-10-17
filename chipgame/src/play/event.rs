
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayEvent {
	PlaySound { sound: chipty::SoundFx },
	PlayMusic { music: Option<chipty::MusicId> },
	PlayLevel,
	Quit,
}
