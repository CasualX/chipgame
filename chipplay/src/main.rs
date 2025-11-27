#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, mem, path, thread, time};
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;

use chipgame::FileSystem;

mod audio;
mod gamepad;

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
	) -> Box<AppStuff> {
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

		Box::new(AppStuff { size, window, surface, context, g, resx })
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
	let time_base = time::Instant::now();

	let config = {
		let config = fs::read_to_string("chipgame.ini").unwrap_or_default();
		chipgame::config::Config::parse(config.as_str())
	};

	// VFS
	let key = paks::Key::default();
	let fs = if let Ok(paks) = paks::FileReader::open("data.paks", &key) {
		FileSystem::Paks(paks, key)
	}
	else {
		FileSystem::StdFs(path::PathBuf::from("data"))
	};

	let mut gamepads = gamepad::GamepadManager::new();

	let mut ap = audio::AudioPlayer::create();
	ap.load(&fs, &config);

	// App state
	let mut app: Option<Box<AppStuff>> = None;
	let mut state = Box::new(chipgame::play::PlayState::default());
	state.lvsets.load();

	let mut key_input = chipcore::Input::default();
	let mut past_now = time_base;

	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let _ = event_loop.run(move |event, elwt| {
		use winit::event::{ElementState, Event, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};

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
					if let Some(app) = app.as_deref_mut() {
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
						PhysicalKey::Code(KeyCode::ArrowLeft) => key_input.left = pressed,
						PhysicalKey::Code(KeyCode::ArrowRight) => key_input.right = pressed,
						PhysicalKey::Code(KeyCode::ArrowUp) => key_input.up = pressed,
						PhysicalKey::Code(KeyCode::ArrowDown) => key_input.down = pressed,
						PhysicalKey::Code(KeyCode::Space) => key_input.a = pressed,
						PhysicalKey::Code(KeyCode::Backspace) => key_input.b = pressed,
						PhysicalKey::Code(KeyCode::Enter) => key_input.start = pressed,
						PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => key_input.select = pressed,
						PhysicalKey::Code(KeyCode::KeyM) if pressed => {
							state.toggle_music();
						}
						PhysicalKey::Code(KeyCode::KeyF) if pressed => {
							if let Some(app) = app.as_deref_mut() {
								let want_fullscreen = app.window.fullscreen().is_none();
								app.set_fullscreen(want_fullscreen);
							}
						}
						PhysicalKey::Code(KeyCode::Escape) if pressed => {
							if let Some(app) = app.as_deref_mut() {
								app.set_fullscreen(false);
							}
						}
						_ => {}
					}
				}
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						let pad_input = gamepads.poll();
						let input = key_input | pad_input;
						state.think(&input);

						app.resx.viewport.maxs = [app.size.width as i32, app.size.height as i32].into();
						app.g.begin();
						let time = time_base.elapsed().as_secs_f64();
						state.draw(&mut app.g, &mut app.resx, time);
						app.g.end();

						for evt in &mem::replace(&mut state.events, Vec::new()) {
							match evt {
								&chipgame::play::PlayEvent::PlaySound { sound } => ap.play_sound(sound),
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
