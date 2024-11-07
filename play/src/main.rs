use std::{fs, mem, thread, time};
use std::collections::HashMap;

#[cfg(windows)]
fn window_builder(size: winit::dpi::PhysicalSize<u32>) -> winit::window::WindowBuilder {
	use winit::platform::windows::WindowBuilderExtWindows;
	winit::window::WindowBuilder::new()
		.with_inner_size(size)
		.with_drag_and_drop(false)
}
#[cfg(not(windows))]
fn window_builder(size: winit::dpi::PhysicalSize<u32>) -> winit::window::WindowBuilder {
	winit::window::WindowBuilder::new()
		.with_inner_size(size)
}

struct AudioPlayer {
	sl: soloud::Soloud,
	sfx: HashMap<chipgame::core::SoundFx, soloud::Wav>,
	music: HashMap<chipgame::MusicId, soloud::Wav>,
	cur_music: Option<(chipgame::MusicId, soloud::Handle)>,
}
impl AudioPlayer {
	fn load_wav(&mut self, fx: chipgame::core::SoundFx, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		wav.load(path).expect("Failed to load sound");
		self.sfx.insert(fx, wav);
	}
	fn load_music(&mut self, music: chipgame::MusicId, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		wav.load(path).expect("Failed to load sound");
		wav.set_looping(true);
		wav.set_volume(0.5);
		self.music.insert(music, wav);
	}
}
impl AudioPlayer {
	fn play(&mut self, sound: chipgame::core::SoundFx) {
		if let Some(wav) = self.sfx.get(&sound) {
			self.sl.play(wav);
		}
	}
	fn play_music(&mut self, music: Option<chipgame::MusicId>) {
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
	let app = clap::command!("play")
		.arg(clap::arg!(-n [n] "Level number to play"))
		.arg(clap::arg!(--dev "Enable developer mode"));
	let matches = app.get_matches();
	let level = if let Some(n) = matches.value_of("n") {
		Some(n.parse::<i32>().expect("Invalid level number"))
	}
	else {
		None
	};
	// let is_dev = matches.is_present("dev");

	let sl = soloud::Soloud::default().expect("Failed to create SoLoud");
	let mut ap = AudioPlayer {
		sl,
		sfx: HashMap::new(),
		music: HashMap::new(),
		cur_music: None,
	};
	ap.load_wav(chipgame::core::SoundFx::GameOver, "data/sfx/death.wav");
	ap.load_wav(chipgame::core::SoundFx::GameWin, "data/sfx/tada.wav");
	ap.load_wav(chipgame::core::SoundFx::Derezz, "data/sfx/derezz.wav");
	ap.load_wav(chipgame::core::SoundFx::ICCollected, "data/sfx/chack.wav");
	ap.load_wav(chipgame::core::SoundFx::KeyCollected, "data/sfx/click.wav");
	ap.load_wav(chipgame::core::SoundFx::BootCollected, "data/sfx/click.wav");
	ap.load_wav(chipgame::core::SoundFx::LockOpened, "data/sfx/door.wav");
	ap.load_wav(chipgame::core::SoundFx::SocketOpened, "data/sfx2/socket unlock.wav");
	ap.load_wav(chipgame::core::SoundFx::CantMove, "data/sfx/oof.wav");
	ap.load_wav(chipgame::core::SoundFx::BlockMoving, "data/sfx/whisk.wav");
	ap.load_wav(chipgame::core::SoundFx::TrapEntered, "data/sfx/traphit.wav");
	ap.load_wav(chipgame::core::SoundFx::BombExplosion, "data/sfx/bomb.wav");
	ap.load_wav(chipgame::core::SoundFx::ButtonPressed, "data/sfx/tick.wav");
	ap.load_wav(chipgame::core::SoundFx::Teleporting, "data/sfx/teleport.wav");
	ap.load_wav(chipgame::core::SoundFx::WallPopup, "data/sfx/popup.wav");
	ap.load_wav(chipgame::core::SoundFx::WaterSplash, "data/sfx/splash.wav");
	ap.load_wav(chipgame::core::SoundFx::BootsStolen, "data/sfx/thief.wav");
	ap.load_wav(chipgame::core::SoundFx::TileEmptied, "data/sfx/whisk.wav");
	ap.load_wav(chipgame::core::SoundFx::BlueWallCleared, "data/sfx2/bump.wav");
	ap.load_wav(chipgame::core::SoundFx::FireWalking, "data/sfx/crackle.wav");
	ap.load_music(chipgame::MusicId::Chip1, "data/music/2Chip1.ogg");
	ap.load_music(chipgame::MusicId::Chip2, "data/music/2Chip2.ogg");
	ap.load_music(chipgame::MusicId::Canyon, "data/music/2Canyon.ogg");

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
	let tileset = shade::png::load(&mut g, Some("scene tiles"), "data/Color_Tileset.png", &shade::png::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, Some(&mut shade::png::gutter(32, 32, 4))).unwrap();
	let tex_info = g.texture2d_get_info(tileset).unwrap();

	let texdigits = shade::png::load(&mut g, Some("digits"), "data/digits.png", &shade::png::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	// Create the shader
	let shader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(shader, include_str!("../../data/standard.vs.glsl"), include_str!("../../data/standard.fs.glsl")) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
	}
	let colorshader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(colorshader, include_str!("../../data/color.vs.glsl"), include_str!("../../data/color.fs.glsl")) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(colorshader).unwrap());
	}
	let uishader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(uishader, include_str!("../../data/ui.vs.glsl"), include_str!("../../data/ui.fs.glsl")) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(uishader).unwrap());
	}

	let mut past_now = time::Instant::now();

	let font = {
		let font: shade::msdfgen::Font = serde_json::from_str(fs::read_to_string("data/font.json").unwrap().as_str()).unwrap();
		let font = Some(font);

		let shader = g.shader_create(None).unwrap();
		if let Err(_) = g.shader_compile(shader, include_str!("../../data/font.vs.glsl"), include_str!("../../data/font.fs.glsl")) {
			panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
		}

		let texture = shade::png::load(&mut g, Some("font"), "data/font.png", &shade::png::TextureProps {
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
	state.load_pack(&format!("{}index.json", chipgame::LEVEL_PACK));
	state.launch();

	if let Some(level) = level {
		state.play_level(level);
	}

	let mut input = chipgame::core::Input::default();

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
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::KeyboardInput { input: keyboard_input, .. }, .. } => {
					match keyboard_input.virtual_keycode {
						Some(winit::event::VirtualKeyCode::Left) => input.left = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::Right) => input.right = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::Up) => input.up = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::Down) => input.down = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::A) => input.a = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::B) => input.b = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::Return) => input.start = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::Space) => input.select = is_pressed(keyboard_input.state),
						Some(winit::event::VirtualKeyCode::M) => if is_pressed(keyboard_input.state) {
							if let Some(fx) = &mut state.fx {
								fx.music_enabled = !fx.music_enabled;
							}
						},
						_ => (),
					}
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		resx.screen_size = [size.width as i32, size.height as i32].into();
		state.think(&input);

		g.begin().unwrap();
		state.draw(&mut g, &resx);
		g.end().unwrap();

		for evt in &mem::replace(&mut state.events, Vec::new()) {
			match evt {
				&chipgame::play::PlayEvent::PlaySound { sound } => ap.play(sound),
				&chipgame::play::PlayEvent::PlayMusic { music } => ap.play_music(music),
				&chipgame::play::PlayEvent::Quit => quit = true,
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
