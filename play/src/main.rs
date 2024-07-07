use std::{fs, thread, time};
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
}
impl AudioPlayer {
	fn load_wav(&mut self, fx: chipgame::core::SoundFx, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		wav.load(path).expect("Failed to load sound");
		self.sfx.insert(fx, wav);
	}
}
impl chipgame::fx::IAudioPlayer for AudioPlayer {
	fn play(&mut self, sound: chipgame::core::SoundFx) {
		if let Some(wav) = self.sfx.get(&sound) {
			self.sl.play(wav);
		}
	}
}

fn main() {
	let app = clap::command!("play")
		.arg(clap::arg!(-n [n] "Level number to play"));
	let matches = app.get_matches();
	let level = if let Some(n) = matches.value_of("n") {
		n.parse::<i32>().expect("Invalid level number")
	}
	else {
		1
	};

	let file_path = format!("data/cc1/level{}.json", level);

	let sl = soloud::Soloud::default().expect("Failed to create SoLoud");
	let mut ap = AudioPlayer { sl, sfx: HashMap::new() };
	ap.load_wav(chipgame::core::SoundFx::GameOver, "data/sfx/death.wav");
	ap.load_wav(chipgame::core::SoundFx::GameWin, "data/sfx/tada.wav");
	ap.load_wav(chipgame::core::SoundFx::Derezz, "data/sfx/derezz.wav");
	ap.load_wav(chipgame::core::SoundFx::ICCollected, "data/sfx/chack.wav");
	ap.load_wav(chipgame::core::SoundFx::KeyCollected, "data/sfx/click.wav");
	ap.load_wav(chipgame::core::SoundFx::BootCollected, "data/sfx/ting.wav");
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

	// Create the shader
	let shader = g.shader_create(None).unwrap();
	if let Err(_) = g.shader_compile(shader, include_str!("../../data/standard.vs.glsl"), include_str!("../../data/standard.fs.glsl")) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
	}

	let mut past_now = time::Instant::now();

	let mut state = chipgame::fx::VisualState::default();
	state.init();
	state.level_index = level;
	state.load_level(&fs::read_to_string(&file_path).unwrap());
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
					if keyboard_input.virtual_keycode == Some(winit::event::VirtualKeyCode::Left) {
						input.left = keyboard_input.state == winit::event::ElementState::Pressed;
					}
					if keyboard_input.virtual_keycode == Some(winit::event::VirtualKeyCode::Right) {
						input.right = keyboard_input.state == winit::event::ElementState::Pressed;
					}
					if keyboard_input.virtual_keycode == Some(winit::event::VirtualKeyCode::Up) {
						input.up = keyboard_input.state == winit::event::ElementState::Pressed;
					}
					if keyboard_input.virtual_keycode == Some(winit::event::VirtualKeyCode::Down) {
						input.down = keyboard_input.state == winit::event::ElementState::Pressed;
					}
					if keyboard_input.virtual_keycode == Some(winit::event::VirtualKeyCode::A) {
						input.a = keyboard_input.state == winit::event::ElementState::Pressed;
					}
					if keyboard_input.virtual_keycode == Some(winit::event::VirtualKeyCode::B) {
						input.b = keyboard_input.state == winit::event::ElementState::Pressed;
					}
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		state.resources = chipgame::fx::Resources {
			tileset,
			tileset_size: [tex_info.width, tex_info.height].into(),
			shader,
			screen_size: [size.width as i32, size.height as i32].into(),
		};
		state.update(&input, Some(&mut ap));
		state.draw(&mut g);

		// Swap the buffers and wait for the next frame
		context.swap_buffers().unwrap();

		// Sleep with a target frame rate of 60 FPS
		let now = time::Instant::now();
		let sleep_dur = time::Duration::from_millis(24).saturating_sub(now - past_now);
		past_now = now;
		thread::sleep(sleep_dur);
	}
}
