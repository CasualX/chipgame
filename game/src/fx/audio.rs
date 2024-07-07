use super::*;

pub trait IAudioPlayer {
	fn play(&mut self, sound: core::SoundFx);
}
