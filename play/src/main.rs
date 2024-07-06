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

fn load_wav(path: &str) -> soloud::Wav {
	use soloud::*;
	let mut wav = Wav::default();
	wav.load(path).expect("Failed to load sound");
	return wav;
}

fn main() {
	let app = clap::command!("play")
		.arg(clap::arg!(<level> "Level file to play"));
	let matches = app.get_matches();
	let file_path = matches.value_of("level").unwrap();

	let sl = soloud::Soloud::default().expect("Failed to create SoLoud");
	let mut sfx = HashMap::new();
	sfx.insert(chipgame::fx::SoundFx::GameOver, load_wav("data/sfx/death.wav"));
	sfx.insert(chipgame::fx::SoundFx::GameWin, load_wav("data/sfx/tada.wav"));
	sfx.insert(chipgame::fx::SoundFx::Derezz, load_wav("data/sfx/derezz.wav"));
	sfx.insert(chipgame::fx::SoundFx::ICCollected, load_wav("data/sfx/chack.wav"));
	sfx.insert(chipgame::fx::SoundFx::KeyCollected, load_wav("data/sfx/click.wav"));
	sfx.insert(chipgame::fx::SoundFx::BootCollected, load_wav("data/sfx/ting.wav"));
	sfx.insert(chipgame::fx::SoundFx::LockOpened, load_wav("data/sfx/door.wav"));
	sfx.insert(chipgame::fx::SoundFx::SocketOpened, load_wav("data/sfx2/socket unlock.wav"));
	sfx.insert(chipgame::fx::SoundFx::CantMove, load_wav("data/sfx/oof.wav"));
	sfx.insert(chipgame::fx::SoundFx::BlockMoving, load_wav("data/sfx/whisk.wav"));
	sfx.insert(chipgame::fx::SoundFx::TrapEntered, load_wav("data/sfx/traphit.wav"));
	sfx.insert(chipgame::fx::SoundFx::BombExplodes, load_wav("data/sfx/bomb.wav"));
	sfx.insert(chipgame::fx::SoundFx::ButtonPressed, load_wav("data/sfx/tick.wav"));
	sfx.insert(chipgame::fx::SoundFx::Teleporting, load_wav("data/sfx/teleport.wav"));
	sfx.insert(chipgame::fx::SoundFx::WallPopup, load_wav("data/sfx/popup.wav"));
	sfx.insert(chipgame::fx::SoundFx::WaterSplash, load_wav("data/sfx/splash.wav"));
	sfx.insert(chipgame::fx::SoundFx::BootsStolen, load_wav("data/sfx/thief.wav"));
	sfx.insert(chipgame::fx::SoundFx::TileEmptied, load_wav("data/sfx/whisk.wav"));
	sfx.insert(chipgame::fx::SoundFx::BlueWallCleared, load_wav("data/sfx2/bump.wav"));

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
		state.update(&input);
		state.draw(&mut g);

		for e in &std::mem::replace(&mut state.events, Vec::new()) {
			println!("Event: {:?}", e);
			match e {
				&chipgame::fx::Event::PlaySound(x) => {
					if let Some(wav) = sfx.get(&x) {
						sl.play(wav);
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
}
