use std::collections::HashMap;
use chipgame::FileSystem;

/// AudioPlayer manages sound effects and music playback using SoLoud.
pub struct AudioPlayer {
	instance: soloud::Soloud,
	sound_effects: HashMap<chipty::SoundFx, soloud::Wav>,
	music_tracks: HashMap<chipty::MusicId, soloud::Wav>,
	active_music: Option<(chipty::MusicId, soloud::Handle)>,
}

impl AudioPlayer {
	/// Creates a new AudioPlayer instance.
	pub fn create() -> AudioPlayer {
		let instance = soloud::Soloud::default().expect("Failed to create SoLoud");
		AudioPlayer {
			instance,
			sound_effects: HashMap::new(),
			music_tracks: HashMap::new(),
			active_music: None,
		}
	}

	/// Deletes the AudioPlayer instance and frees resources.
	#[cfg(windows)]
	pub fn delete(self) {
		let AudioPlayer {
			instance,
			sound_effects,
			music_tracks,
			active_music: _,
		} = self;
		drop(sound_effects);
		drop(music_tracks);
		unsafe { soloud::Soloud::delete(instance) };
	}

	/// Loads sound effects and music tracks from the given configuration.
	pub fn load(&mut self, fs: &FileSystem, config: &chipgame::config::Config) {
		for (fx, path) in &config.sound_fx {
			self.load_wav(*fx, fs, path);
		}
		for (id, path) in &config.music {
			self.load_music(*id, fs, path);
		}
	}
	fn load_wav(&mut self, fx: chipty::SoundFx, fs: &FileSystem, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		let data = fs.read(path).expect("Failed to read sound file");
		wav.load_mem(&data).expect("Failed to load sound");
		self.sound_effects.insert(fx, wav);
	}
	fn load_music(&mut self, music: chipty::MusicId, fs: &FileSystem, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		let data = fs.read(path).expect("Failed to read music file");
		wav.load_mem(&data).expect("Failed to load music");
		wav.set_looping(true);
		wav.set_volume(0.375);
		self.music_tracks.insert(music, wav);
	}

	/// Plays a sound effect.
	pub fn play_sound(&mut self, sound: chipty::SoundFx) {
		if let Some(wav) = self.sound_effects.get(&sound) {
			self.instance.play(wav);
		}
	}
	/// Changes the currently playing music track.
	pub fn play_music(&mut self, music: Option<chipty::MusicId>) {
		if self.active_music.map(|(music, _)| music) != music {
			if let Some((_, handle)) = self.active_music {
				self.instance.stop(handle);
			}
			self.active_music = None;
			if let Some(music) = music {
				if let Some(wav) = self.music_tracks.get(&music) {
					let handle = self.instance.play(wav);
					self.active_music = Some((music, handle));
				}
			}
		}
	}
}
