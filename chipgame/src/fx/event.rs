
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FxEvent {
	PlaySound { sound: chipty::SoundFx },
	PlayMusic { music: Option<chipty::MusicId> },
	Pause,
	Scout,
	Unpause,
	GameWin,
	GameOver,
}
