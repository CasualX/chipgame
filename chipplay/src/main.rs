#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{mem, thread, time};
use std::collections::HashMap;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;

use chipgame::FileSystem;

mod gamepad;

struct AudioPlayer {
	sl: soloud::Soloud,
	sfx: HashMap<chipty::SoundFx, soloud::Wav>,
	music: HashMap<chipty::MusicId, soloud::Wav>,
	cur_music: Option<(chipty::MusicId, soloud::Handle)>,
}
impl AudioPlayer {
	fn load_wav(&mut self, fx: chipty::SoundFx, fs: &FileSystem, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		let data = fs.read(path).expect("Failed to read sound file");
		wav.load_mem(&data).expect("Failed to load sound");
		self.sfx.insert(fx, wav);
	}
	fn load_music(&mut self, music: chipty::MusicId, fs: &FileSystem, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		let data = fs.read(path).expect("Failed to read music file");
		wav.load_mem(&data).expect("Failed to load music");
		wav.set_looping(true);
		wav.set_volume(0.375);
		self.music.insert(music, wav);
	}
}
impl AudioPlayer {
	fn play(&mut self, sound: chipty::SoundFx) {
		if let Some(wav) = self.sfx.get(&sound) {
			self.sl.play(wav);
		}
	}
	fn play_music(&mut self, music: Option<chipty::MusicId>) {
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

fn load_audio(fs: &FileSystem, config: &chipgame::config::Config, ap: &mut AudioPlayer) {
	for (fx, path) in &config.sound_fx {
		ap.load_wav(*fx, fs, path);
	}
	for (id, path) in &config.music {
		ap.load_music(*id, fs, path);
	}
}

struct AppStuff {
	size: winit::dpi::PhysicalSize<u32>,
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	resx: chipgame::fx::Resources,
}

impl AppStuff {
	fn new(
		elwt: &winit::event_loop::EventLoopWindowTarget<()>,
		fs: &FileSystem,
		config: &chipgame::config::Config,
	) -> AppStuff {
		use glutin::config::ConfigTemplateBuilder;
		use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
		use glutin::display::GetGlDisplay;
		use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
		use raw_window_handle::HasRawWindowHandle;

		let template = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let size = winit::dpi::PhysicalSize::new(800, 600);

		#[cfg(windows)]
		let window_builder = {
			use winit::platform::windows::WindowBuilderExtWindows;
			winit::window::WindowBuilder::new()
				.with_title("Play ChipGame")
				.with_inner_size(size)
				.with_drag_and_drop(false)
		};

		#[cfg(not(windows))]
		let window_builder = winit::window::WindowBuilder::new()
			.with_title("Play ChipGame")
			.with_inner_size(size);

		let (window, gl_config) = glutin_winit::DisplayBuilder::new()
			.with_window_builder(Some(window_builder))
			.build(elwt, template, |configs| configs.max_by_key(|c| c.num_samples()).unwrap())
			.expect("Failed to build window and GL config");

		let window = window.expect("DisplayBuilder did not build a Window");
		let raw_window_handle = window.raw_window_handle();

		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
			.build(Some(raw_window_handle));

		let gl_display = gl_config.display();

		let not_current = unsafe {
			gl_display.create_context(&gl_config, &context_attributes)
		}.expect("Failed to create GL context");

		let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
			raw_window_handle,
			NonZeroU32::new(size.width.max(1)).unwrap(),
			NonZeroU32::new(size.height.max(1)).unwrap(),
		);

		let surface = unsafe {
			gl_display.create_window_surface(&gl_config, &attrs)
		}.expect("Failed to create GL surface");

		let context = not_current
			.make_current(&surface)
			.expect("Failed to make GL context current");

		// Load GL function pointers
		shade::gl::capi::load_with(|s| {
			let c = CString::new(s).unwrap();
			gl_display.get_proc_address(&c)
		});

		// Now that GL is ready, create graphics and resources
		let mut g = shade::gl::GlGraphics::new();
		let mut resx = chipgame::fx::Resources::default();
		resx.load(fs, config, &mut g);

		AppStuff { size, window, surface, context, g, resx }
	}

	fn set_title(&self, state: &chipgame::play::PlayState) {
		if let Some(fx) = &state.fx {
			self.window.set_title(&format!("{} - Level {} - {}", state.lvsets.current().title, fx.level_number, fx.gs.field.name));
		}
		else if let Some(level_set) = state.lvsets.collection.get(state.lvsets.selected) {
			self.window.set_title(&level_set.title);
		}
		else {
			self.window.set_title("Play ChipGame");
		}
	}

	fn set_fullscreen(&self, fullscreen: bool) {
		// Borderless fullscreen on the current monitor; hide cursor when fullscreen
		let target = if fullscreen {
			let monitor = self.window.current_monitor();
			Some(winit::window::Fullscreen::Borderless(monitor))
		}
		else {
			None
		};
		self.window.set_fullscreen(target);
		self.window.set_cursor_visible(!fullscreen);
	}
}

fn main() {
	let key = paks::Key::default();
	let fs = if let Ok(paks) = paks::FileReader::open("data.paks", &key) {
		FileSystem::Paks(paks, key)
	}
	else {
		FileSystem::StdFs(std::path::PathBuf::from("data"))
	};

	let mut gamepads = gamepad::GamepadManager::new();

	let sl = soloud::Soloud::default().expect("Failed to create SoLoud");
	let mut ap = AudioPlayer { sl, sfx: HashMap::new(), music: HashMap::new(), cur_music: None };

	let config = {
		let config = std::fs::read_to_string("chipgame.ini").unwrap_or_default();
		chipgame::config::Config::parse(config.as_str())
	};
	load_audio(&fs, &config, &mut ap);

	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	// App state to be initialized on Event::Resumed
	let mut app: Option<AppStuff> = None;

	let mut kbd_input = chipcore::Input::default();
	let time_base = time::Instant::now();
	let mut past_now = time::Instant::now();

	let mut state = chipgame::play::PlayState::default();
	state.lvsets.load();

	use winit::event::{ElementState, Event, WindowEvent};
	use winit::keyboard::{KeyCode, PhysicalKey};

	let _ = event_loop.run(move |event, elwt| {
		match event {
			Event::Resumed => {
				if app.is_none() {
					let mut built = AppStuff::new(elwt, &fs, &config);
					state.launch(&mut built.g);
					app = Some(built);
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(new_size) => {
					if let Some(app) = &mut app {
						let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
						let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
						app.size = new_size;
						app.surface.resize(&app.context, width, height);
					}
				}
				WindowEvent::CloseRequested => elwt.exit(),
				WindowEvent::KeyboardInput { event, .. } => {
					let pressed = matches!(event.state, ElementState::Pressed);

					match event.physical_key {
						PhysicalKey::Code(KeyCode::ArrowLeft) => kbd_input.left = pressed,
						PhysicalKey::Code(KeyCode::ArrowRight) => kbd_input.right = pressed,
						PhysicalKey::Code(KeyCode::ArrowUp) => kbd_input.up = pressed,
						PhysicalKey::Code(KeyCode::ArrowDown) => kbd_input.down = pressed,
						PhysicalKey::Code(KeyCode::Space) => kbd_input.a = pressed,
						PhysicalKey::Code(KeyCode::Backspace) => kbd_input.b = pressed,
						PhysicalKey::Code(KeyCode::Enter) => kbd_input.start = pressed,
						PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => kbd_input.select = pressed,
						PhysicalKey::Code(KeyCode::KeyM) if pressed => {
							state.toggle_music();
						}
						PhysicalKey::Code(KeyCode::KeyF) if pressed => {
							if let Some(app) = &mut app {
								let want_fullscreen = app.window.fullscreen().is_none();
								app.set_fullscreen(want_fullscreen);
							}
						}
						PhysicalKey::Code(KeyCode::Escape) if pressed => {
							if let Some(app) = &mut app {
								app.set_fullscreen(false);
							}
						}
						_ => {}
					}
				}
				WindowEvent::RedrawRequested => {
					if let Some(app) = &mut app {
						let pad_input = gamepads.poll();
						let input = kbd_input | pad_input;
						state.think(&input);

						app.resx.viewport.maxs = [app.size.width as i32, app.size.height as i32].into();
						app.g.begin();
						let time = time_base.elapsed().as_secs_f64();
						state.draw(&mut app.g, &mut app.resx, time);
						app.g.end();

						for evt in &mem::replace(&mut state.events, Vec::new()) {
							match evt {
								&chipgame::play::PlayEvent::PlaySound { sound } => ap.play(sound),
								&chipgame::play::PlayEvent::PlayMusic { music } => ap.play_music(music),
								&chipgame::play::PlayEvent::Quit => elwt.exit(),
								&chipgame::play::PlayEvent::PlayLevel => app.set_title(&state),
							}
						}

						app.surface.swap_buffers(&app.context).unwrap();
					}

					let now = time::Instant::now();
					let sleep_dur = time::Duration::from_millis(24).saturating_sub(now - past_now);
					past_now = now;
					if sleep_dur > time::Duration::ZERO {
						thread::sleep(sleep_dur);
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_ref() {
					app.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
