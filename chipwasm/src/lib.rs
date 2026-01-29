use std::{mem, ptr};

mod api;

const CHIPGAME_INI: &str = include_str!("../../chipgame.webgl.ini");
const DATA_PAK: &[u8] = include_bytes!("../../target/publish/data.paks");
const CCLP1_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp1.paks");
const CCLP2_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp2.paks");
const CCLP3_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp3.paks");
const CCLP4_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp4.paks");
const CCLP5_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp5.paks");

fn play_sound(sound: chipty::SoundFx) {
	unsafe {
		api::play_sound(sound as i32);
	}
}
fn play_music(music: Option<chipty::MusicId>) {
	let id = music.map(|m| m as i32).unwrap_or(-1);
	unsafe {
		api::play_music(id);
	}
}
fn set_title(state: &chipgame::play::PlayState) {
	let title = if let Some(fx) = &state.fx {
		format!("{} - Level {} - {}", state.lvsets.current().title, fx.level_number, fx.game.field.name)
	}
	else if let Some(level_set) = state.lvsets.collection.get(state.lvsets.selected as usize) {
		level_set.title.clone()
	}
	else {
		"Choose LevelSet".to_string()
	};
	unsafe {
		api::set_title(title.as_ptr(), title.len());
	}
}
fn quit_game() {
	unsafe {
		api::quit_game();
	}
}

pub struct Instance {
	graphics: shade::webgl::WebGLGraphics,
	resx: chipgame::fx::Resources,
	play: chipgame::play::PlayState,
}

fn load_levelset(data: &[u8], name: String, play: &mut chipgame::play::PlayState) {
	let key = paks::Key::default();
	let paks = paks::MemoryReader::from_bytes(data, &key).expect("Failed to open levelset paks");
	let fs = chipgame::FileSystem::Memory(paks, key.clone());
	chipgame::play::load_levelset(&fs, name, &mut play.lvsets.collection);
}

#[no_mangle]
pub extern "C" fn create() -> *mut Instance {
	shade::webgl::setup_panic_hook();

	let mut instance = Box::new(Instance {
		graphics: shade::webgl::WebGLGraphics::new(shade::webgl::WebGLConfig {
			srgb: false,
		}),
		resx: chipgame::fx::Resources::default(),
		play: chipgame::play::PlayState::default(),
	});

	// Load the resources
	let config = chipgame::config::Config::parse(CHIPGAME_INI);
	let key = paks::Key::default();
	let paks = paks::MemoryReader::from_bytes(DATA_PAK, &key).expect("Failed to open data.paks");
	let fs = chipgame::FileSystem::Memory(paks, key.clone());
	instance.resx.load(&fs, &config, instance.graphics.as_graphics());
	register_audio_assets(&fs, &config);

	load_levelset(CCLP1_PAK, "cclp1".to_string(), &mut instance.play);
	load_levelset(CCLP2_PAK, "cclp2".to_string(), &mut instance.play);
	load_levelset(CCLP3_PAK, "cclp3".to_string(), &mut instance.play);
	load_levelset(CCLP4_PAK, "cclp4".to_string(), &mut instance.play);
	load_levelset(CCLP5_PAK, "cclp5".to_string(), &mut instance.play);

	instance.play.launch(instance.graphics.as_graphics());

	Box::into_raw(instance)
}

fn register_audio_assets(fs: &chipgame::FileSystem, config: &chipgame::config::Config) {
	for (&fx, path) in &config.sound_fx {
		if let Ok(data) = fs.read(path) {
			unsafe {
				api::register_sound(fx as i32, data.as_ptr(), data.len());
			}
		}
	}
	for (&music, path) in &config.music {
		if let Ok(data) = fs.read(path) {
			unsafe {
				api::register_music(music as i32, data.as_ptr(), data.len());
			}
		}
	}
}


#[no_mangle]
pub extern "C" fn destroy(instance: *mut Instance) {
	_ = unsafe { Box::from_raw(instance) };
}

#[no_mangle]
pub extern "C" fn think(instance: *mut Instance, buttons: u8) {
	let instance = unsafe { &mut *instance };
	let input = chipcore::Input::decode(buttons);
	instance.play.think(&input);

	for evt in &mem::replace(&mut instance.play.events, Vec::new()) {
		match evt {
			&chipgame::play::PlayEvent::PlaySound { sound } => play_sound(sound),
			&chipgame::play::PlayEvent::PlayMusic { music } => play_music(music),
			&chipgame::play::PlayEvent::SetTitle => set_title(&instance.play),
			&chipgame::play::PlayEvent::Restart => instance.play.launch(instance.graphics.as_graphics()),
			&chipgame::play::PlayEvent::Quit => quit_game(),
		}
	}
}

#[no_mangle]
pub extern "C" fn draw(instance: *mut Instance, time: f64, width: i32, height: i32) {
	let instance = unsafe { &mut *instance };
	let g = instance.graphics.as_graphics();
	instance.resx.viewport.maxs = cvmath::Vec2i(width, height);
	instance.play.draw(g, &instance.resx, time);
}

#[no_mangle]
extern "C" fn chipgame_write_file(path_ptr: *const u8, path_len: usize, content_ptr: *const u8, content_len: usize) -> i32 {
	unsafe {
		api::write_file(path_ptr, path_len, content_ptr, content_len)
	}
}

#[no_mangle]
extern "C" fn chipgame_read_file(path_ptr: *const u8, path_len: usize, content_ptr: *mut String) -> i32 {
	unsafe {
		let mut size: usize = 0;
		if api::read_file(path_ptr, path_len, ptr::null_mut(), &mut size as *mut usize) != 0 {
			return -1;
		}
		let content = &mut *content_ptr;
		content.as_mut_vec().set_len(0);
		content.reserve(size);
		let mut read = size;
		if api::read_file(path_ptr, path_len, content.as_mut_ptr() as *mut u8, &mut read as *mut usize) != 0 {
			return -1;
		}
		content.as_mut_vec().set_len(read);
		return 0;
	}
}
