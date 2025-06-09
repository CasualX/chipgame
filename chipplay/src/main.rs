#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, mem, thread, time};
use std::collections::HashMap;

mod xinput;

#[cfg(windows)]
fn window_builder(size: winit::dpi::PhysicalSize<u32>) -> winit::window::WindowBuilder {
	use winit::platform::windows::WindowBuilderExtWindows;
	winit::window::WindowBuilder::new()
		.with_title("Play ChipGame")
		.with_inner_size(size)
		.with_drag_and_drop(false)
}
#[cfg(not(windows))]
fn window_builder(size: winit::dpi::PhysicalSize<u32>) -> winit::window::WindowBuilder {
	winit::window::WindowBuilder::new()
		.with_title("Play ChipGame")
		.with_inner_size(size)
}

struct AudioPlayer {
	sl: soloud::Soloud,
	sfx: HashMap<chipcore::SoundFx, soloud::Wav>,
	music: HashMap<chipgame::data::MusicId, soloud::Wav>,
	cur_music: Option<(chipgame::data::MusicId, soloud::Handle)>,
}
impl AudioPlayer {
	fn load_wav(&mut self, fx: chipcore::SoundFx, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		wav.load(path).expect("Failed to load sound");
		self.sfx.insert(fx, wav);
	}
	fn load_music(&mut self, music: chipgame::data::MusicId, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		wav.load(path).expect("Failed to load sound");
		wav.set_looping(true);
		wav.set_volume(0.5);
		self.music.insert(music, wav);
	}
}
impl AudioPlayer {
	fn play(&mut self, sound: chipcore::SoundFx) {
		if let Some(wav) = self.sfx.get(&sound) {
			self.sl.play(wav);
		}
	}
	fn play_music(&mut self, music: Option<chipgame::data::MusicId>) {
		if self.cur_music.map(|(music, _)| music) != music {
			if let Some((_, handle)) = self.cur_music {
				self.sl.stop(handle);
			}
			self.cur_music = None;
			if let Some(music) = music {
				if let Some(wav) = self.music.get(&music) {
					let handle = self.sl.play(wav);
					self.cur_music = Some((music, handle));
				}
			}
		}
	}
}

fn main() {
	// let app = clap::command!("play")
	// 	.arg(clap::arg!(--"level-pack" -p [level_pack] "Level pack to play"))
	// 	.arg(clap::arg!(-n [n] "Level number to play"))
	// 	.arg(clap::arg!(--dev "Enable developer mode"));
	// let matches = app.get_matches();
	// let level_pack = matches.value_of("level-pack").expect("Level pack not specified");
	// let level_pack = std::path::Path::new(level_pack);
	// let level = if let Some(n) = matches.value_of("n") {
	// 	Some(n.parse::<i32>().expect("Invalid level number"))
	// }
	// else {
	// 	None
	// };
	// let is_dev = matches.is_present("dev");

	let xinput = xinput::XInput::new();

	let sl = soloud::Soloud::default().expect("Failed to create SoLoud");
	let mut ap = AudioPlayer {
		sl,
		sfx: HashMap::new(),
		music: HashMap::new(),
		cur_music: None,
	};
	ap.load_wav(chipcore::SoundFx::GameOver, "data/sfx/death.wav");
	ap.load_wav(chipcore::SoundFx::GameWin, "data/sfx/tada.wav");
	ap.load_wav(chipcore::SoundFx::Derezz, "data/sfx/derezz.wav");
	ap.load_wav(chipcore::SoundFx::ICCollected, "data/sfx/chack.wav");
	ap.load_wav(chipcore::SoundFx::KeyCollected, "data/sfx/click.wav");
	ap.load_wav(chipcore::SoundFx::BootCollected, "data/sfx/click.wav");
	ap.load_wav(chipcore::SoundFx::LockOpened, "data/sfx/door.wav");
	ap.load_wav(chipcore::SoundFx::SocketOpened, "data/sfx2/socket unlock.wav");
	ap.load_wav(chipcore::SoundFx::CantMove, "data/sfx/oof.wav");
	ap.load_wav(chipcore::SoundFx::BlockMoving, "data/sfx/whisk.wav");
	ap.load_wav(chipcore::SoundFx::TrapEntered, "data/sfx/traphit.wav");
	ap.load_wav(chipcore::SoundFx::BombExplosion, "data/sfx/bomb.wav");
	ap.load_wav(chipcore::SoundFx::ButtonPressed, "data/sfx/tick.wav");
	ap.load_wav(chipcore::SoundFx::Teleporting, "data/sfx/teleport.wav");
	ap.load_wav(chipcore::SoundFx::WallPopup, "data/sfx/popup.wav");
	ap.load_wav(chipcore::SoundFx::WaterSplash, "data/sfx/splash.wav");
	ap.load_wav(chipcore::SoundFx::BootsStolen, "data/sfx/thief.wav");
	ap.load_wav(chipcore::SoundFx::TileEmptied, "data/sfx/whisk.wav");
	ap.load_wav(chipcore::SoundFx::BlueWallCleared, "data/sfx2/bump.wav");
	ap.load_wav(chipcore::SoundFx::FireWalking, "data/sfx/crackle.wav");
	ap.load_wav(chipcore::SoundFx::CursorMove, "data/sfxui/click-buttons-ui-menu.mp3");
	ap.load_wav(chipcore::SoundFx::CursorSelect, "data/sfx/bump.wav");
	ap.load_music(chipgame::data::MusicId::Chip1, "data/music/2Chip1.ogg");
	ap.load_music(chipgame::data::MusicId::Chip2, "data/music/2Chip2.ogg");
	ap.load_music(chipgame::data::MusicId::Canyon, "data/music/2Canyon.ogg");

	let mut size = winit::dpi::PhysicalSize::new(800, 600);

	let mut event_loop = winit::event_loop::EventLoop::new();

	let window_context = glutin::ContextBuilder::new()
		.with_multisampling(4)
		.build_windowed(window_builder(size), &event_loop)
		.unwrap();

	let context = unsafe { window_context.make_current().unwrap() };

	shade::gl::capi::load_with(|s| context.get_proc_address(s) as *const _);

	// Create the graphics context
	let mut g = shade::gl::GlGraphics::new();

	// Load the texture
	let tileset = shade::image::png::load_file(&mut g, Some("scene tiles"), "data/Color_Tileset.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, Some(&mut shade::image::gutter(32, 32))).unwrap();
	let tex_info = g.texture2d_get_info(tileset).unwrap();

	let texdigits = shade::image::png::load_file(&mut g, Some("digits"), "data/digits.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	// Create the shader
	let shader = g.shader_create(None, include_str!("../../data/standard.vs.glsl"), include_str!("../../data/standard.fs.glsl")).unwrap();
	let colorshader = g.shader_create(None, include_str!("../../data/color.vs.glsl"), include_str!("../../data/color.fs.glsl")).unwrap();
	let uishader = g.shader_create(None, include_str!("../../data/ui.vs.glsl"), include_str!("../../data/ui.fs.glsl")).unwrap();

	let mut past_now = time::Instant::now();

	let font = {
		let font: shade::msdfgen::Font = serde_json::from_str(fs::read_to_string("data/font.json").unwrap().as_str()).unwrap();
		let font = Some(font);

		let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS).unwrap();

		let texture = shade::image::png::load_file(&mut g, Some("font"), "data/font.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		shade::d2::FontResource { font, shader, texture }
	};

	let mut resx = chipgame::fx::Resources {
		tileset,
		tileset_size: [tex_info.width, tex_info.height].into(),
		shader,
		screen_size: [size.width as i32, size.height as i32].into(),
		colorshader,
		uishader,
		texdigits,
		font,
	};
	let mut state = chipgame::play::PlayState::default();
	state.lvsets.load();
	state.launch(&mut g);

	let mut kbd_input = chipcore::Input::default();

	// Main loop
	let mut quit = false;
	while !quit {
		// Handle events
		use winit::platform::run_return::EventLoopExtRunReturn as _;
		event_loop.run_return(|event, _, control_flow| {
			*control_flow = winit::event_loop::ControlFlow::Wait;

			// if let winit::event::Event::WindowEvent { event, .. } = &event {
			// 	// Print only Window events to reduce noise
			// 	println!("{:?}", event);
			// }

			match event {
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
					size = new_size;
					context.resize(new_size);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
					quit = true;
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::KeyboardInput { input, .. }, .. } => {
					let left = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Left)) || input.scancode == 0x1e;
					let right = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Right)) || input.scancode == 0x20;
					let up = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Up)) || input.scancode == 0x11;
					let down = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Down)) || input.scancode == 0x1f;

					let a = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Space));
					let b = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Back));

					let start = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Return));
					let select = matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::Escape));

					if left { kbd_input.left = is_pressed(input.state); }
					if right { kbd_input.right = is_pressed(input.state); }
					if up { kbd_input.up = is_pressed(input.state); }
					if down { kbd_input.down = is_pressed(input.state); }
					if a { kbd_input.a = is_pressed(input.state); }
					if b { kbd_input.b = is_pressed(input.state); }
					if start { kbd_input.start = is_pressed(input.state); }
					if select { kbd_input.select = is_pressed(input.state); }

					if matches!(input.virtual_keycode, Some(winit::event::VirtualKeyCode::M)) && is_pressed(input.state) {
						state.toggle_music();
					}
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		let mut x_input = chipcore::Input::default();
		xinput.get_state(&mut x_input);

		resx.screen_size = [size.width as i32, size.height as i32].into();
		let input = kbd_input | x_input;
		state.think(&input);

		g.begin().unwrap();
		state.draw(&mut g, &resx);
		g.end().unwrap();

		for evt in &mem::replace(&mut state.events, Vec::new()) {
			match evt {
				&chipgame::play::PlayEvent::PlaySound { sound } => ap.play(sound),
				&chipgame::play::PlayEvent::PlayMusic { music } => ap.play_music(music),
				&chipgame::play::PlayEvent::Quit => quit = true,
				&chipgame::play::PlayEvent::PlayLevel => {
					if let Some(fx) = &state.fx {
						context.window().set_title(&format!("{} - Level {} - {}", state.lvsets.current().name, fx.level_number, fx.gs.field.name));
					}
					else if let Some(level_pack) = state.lvsets.collection.get(state.lvsets.selected) {
						context.window().set_title(&level_pack.title);
					}
					else {
						context.window().set_title("Play ChipGame");
					}
				}
			}
		}

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();

		// Sleep with a target frame rate of 60 FPS
		let now = time::Instant::now();
		let sleep_dur = time::Duration::from_millis(24).saturating_sub(now - past_now);
		past_now = now;
		thread::sleep(sleep_dur);
	}

	// App crashes when dropping soloud...
	std::mem::forget(ap.sl);
}

fn is_pressed(state: winit::event::ElementState) -> bool {
	match state {
		winit::event::ElementState::Pressed => true,
		winit::event::ElementState::Released => false,
	}
}
