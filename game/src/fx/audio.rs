use super::*;

pub trait IAudioPlayer {
	fn play(&mut self, sound: core::SoundFx);
	fn play_music(&mut self, music: Option<MusicId>);
}
