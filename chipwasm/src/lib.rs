use std::mem;

mod api;

const CHIPGAME_INI: &str = include_str!("../../chipgame.webgl.ini");
const DATA_PAK: &[u8] = include_bytes!("../../target/publish/data.paks");
const CCLP1_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp1.paks");
const CCLP2_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp2.paks");
const CCLP3_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp3.paks");
const CCLP4_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp4.paks");
const CCLP5_PAK: &[u8] = include_bytes!("../../target/publish/levelsets/cclp5.paks");

fn play_sound(sound: chipty::SoundFx) {
	let sound_json = serde_json::to_string(&sound).unwrap();
	unsafe {
		api::play_sound(sound_json.as_ptr(), sound_json.len());
	}
}
fn play_music(music: Option<chipty::MusicId>) {
	let music_json = serde_json::to_string(&music).unwrap();
	unsafe {
		api::play_music(music_json.as_ptr(), music_json.len());
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

	load_levelset(CCLP1_PAK, "cclp1".to_string(), &mut instance.play);
	load_levelset(CCLP2_PAK, "cclp2".to_string(), &mut instance.play);
	load_levelset(CCLP3_PAK, "cclp3".to_string(), &mut instance.play);
	load_levelset(CCLP4_PAK, "cclp4".to_string(), &mut instance.play);
	load_levelset(CCLP5_PAK, "cclp5".to_string(), &mut instance.play);

	instance.play.launch(instance.graphics.as_graphics());

	Box::into_raw(instance)
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
