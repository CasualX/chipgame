#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, path, time};
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;

use chipgame::editor;
use chipgame::FileSystem;

mod audio;
mod gamepad;

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;
const FRAME_TIME: time::Duration = time::Duration::from_nanos((NANOS_PER_SECOND / chipcore::FPS as f64) as u64);
const SLOW_THRESHOLD: time::Duration = time::Duration::from_nanos((NANOS_PER_SECOND / (chipcore::FPS + 1) as f64) as u64);

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
		use glutin::surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface};
		use raw_window_handle::HasRawWindowHandle;

		let template = ConfigTemplateBuilder::new()
			.with_alpha_size(8)
			.with_multisampling(4);

		let size = winit::dpi::PhysicalSize::new(800, 600);

		#[cfg(windows)]
		let window_builder = {
			use winit::platform::windows::WindowBuilderExtWindows;
			winit::window::WindowBuilder::new()
				.with_title("ChipEdit")
				.with_inner_size(size)
				.with_drag_and_drop(false)
		};
		#[cfg(not(windows))]
		let window_builder = winit::window::WindowBuilder::new()
			.with_title("ChipEdit")
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

		if config.vsync {
			if let Err(err) = surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())) {
				eprintln!("Failed to enable vsync: {err}");
			}
		}

		// Load GL functions for shaders/textures
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

	fn set_fullscreen(&self, fullscreen: bool) {
		// Borderless fullscreen on the current monitor
		let target = if fullscreen {
			let monitor = self.window.current_monitor();
			Some(winit::window::Fullscreen::Borderless(monitor))
		}
		else {
			None
		};
		self.window.set_fullscreen(target);
	}
}

fn load_level(editor: &mut editor::EditorState, file_path: &Option<path::PathBuf>, app: Option<&AppStuff>) {
	if let Some(fp) = file_path {
		match fs::read_to_string(fp) {
			Ok(contents) => {
				editor.load_level(&contents);
				if let Some(app) = app {
					app.window.set_title(&format!("ChipEdit - {}", fp.display()));
				}
			}
			Err(err) => {
				eprintln!("Failed to load level {}: {err}", fp.display());
			}
		}
	}
}

fn main() {
	let time_base = time::Instant::now();

	// CLI: optional level path
	let matches = clap::command!()
		.arg(clap::arg!([level] "Level file to open").value_parser(clap::value_parser!(path::PathBuf)))
		.get_matches();

	let mut file_path = matches.get_one::<path::PathBuf>("level").cloned();

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
	let mut editor = editor::EditorState::default();
	editor.load_level(include_str!("template.json"));

	let mut current_tool = None;
	let mut shift_held = false;
	let mut key_left = false;
	let mut key_right = false;
	let mut key_up = false;
	let mut key_down = false;
	let mut gamepad_start = false;
	let mut music_enabled = true;
	let mut last_frame_start = time::Instant::now();
	let mut tick_budget = time::Duration::ZERO;

	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	let _ = event_loop.run(move |event, elwt| {
		use winit::event::{ElementState, Event, MouseButton, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};
		use winit::window::CursorIcon;

		match event {
			Event::Resumed => {
				if app.is_none() {
					let built = AppStuff::new(elwt, &fs, &config);

					// Window title and initial level
					built.window.set_title("ChipEdit - (unsaved)");
					app = Some(built);

					load_level(&mut editor, &file_path, app.as_deref());
					last_frame_start = time::Instant::now() - FRAME_TIME;
					tick_budget = time::Duration::ZERO;
				}
			}
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::ModifiersChanged(new_mods) => {
					shift_held = new_mods.state().shift_key();
				}
				WindowEvent::Resized(new_size) => {
					if let Some(app) = app.as_deref_mut() {
						let width = NonZeroU32::new(new_size.width.max(1)).unwrap();
						let height = NonZeroU32::new(new_size.height.max(1)).unwrap();
						app.size = new_size;
						app.surface.resize(&app.context, width, height);
						app.resx.viewport.maxs = [app.size.width as i32, app.size.height as i32].into();
					}
					editor.set_screen_size(new_size.width as i32, new_size.height as i32);
				}
				WindowEvent::CloseRequested => elwt.exit(),
				WindowEvent::KeyboardInput { event, .. } => {
					let pressed = matches!(event.state, ElementState::Pressed);

					match event.physical_key {
						PhysicalKey::Code(KeyCode::ArrowLeft) => key_left = pressed,
						PhysicalKey::Code(KeyCode::ArrowRight) => key_right = pressed,
						PhysicalKey::Code(KeyCode::ArrowUp) => key_up = pressed,
						PhysicalKey::Code(KeyCode::ArrowDown) => key_down = pressed,
						PhysicalKey::Code(KeyCode::Enter) if pressed => editor.toggle_play(),
						PhysicalKey::Code(KeyCode::Delete) => editor.delete(pressed),
						PhysicalKey::Code(KeyCode::F2) if pressed => {
							// Open file dialog to load a level
							let mut dialog = rfd::FileDialog::new()
								.add_filter("Level", &["json"])
								.set_title("Open Level");
							if let Some(existing) = &file_path {
								if let Some(parent) = existing.parent() {
									dialog = dialog.set_directory(parent);
								}
							}
							if let Some(path) = dialog.pick_file() {
								file_path = Some(path);
								load_level(&mut editor, &file_path, app.as_deref());
							}
						}
						PhysicalKey::Code(KeyCode::F5) if pressed => {
							let mut dialog = rfd::FileDialog::new()
								.add_filter("Level", &["json"])
								.set_title("Save Level");
							if let Some(existing) = &file_path {
								if let Some(parent) = existing.parent() {
									dialog = dialog.set_directory(parent);
								}
								if let Some(name) = existing.file_name().and_then(|s| s.to_str()) {
									dialog = dialog.set_file_name(name);
								}
							}
							if let Some(path) = dialog.save_file() {
								let contents = editor.save_level();
								match fs::write(&path, contents) {
									Ok(_) => {
										file_path = Some(path);
										if let Some(app) = &app {
											app.window.set_title(&format!("ChipEdit - {}", file_path.as_ref().unwrap().display()));
										}
									}
									Err(err) => {
										eprintln!("Failed to save level: {err}");
									}
								}
							}
						}
						PhysicalKey::Code(KeyCode::KeyT) => editor.tool_terrain(pressed),
						PhysicalKey::Code(KeyCode::KeyE) => editor.tool_entity(pressed),
						PhysicalKey::Code(KeyCode::KeyC) => editor.tool_connection(pressed),
						PhysicalKey::Code(KeyCode::ShiftLeft | KeyCode::ShiftRight) => editor.key_shift(pressed),
						PhysicalKey::Code(KeyCode::Numpad8) if pressed => {
							if shift_held { editor.expand_top(); }
							else { editor.crop_top(); }
							let level = editor.save_level();
							editor.reload_level(&level);
						}
						PhysicalKey::Code(KeyCode::Numpad2) if pressed => {
							if shift_held { editor.expand_bottom(); }
							else { editor.crop_bottom(); }
							let level = editor.save_level();
							editor.reload_level(&level);
						}
						PhysicalKey::Code(KeyCode::Numpad4) if pressed => {
							if shift_held { editor.expand_left(); }
							else { editor.crop_left(); }
							let level = editor.save_level();
							editor.reload_level(&level);
						}
						PhysicalKey::Code(KeyCode::Numpad6) if pressed => {
							if shift_held { editor.expand_right(); }
							else { editor.crop_right(); }
							let level = editor.save_level();
							editor.reload_level(&level);
						}
						PhysicalKey::Code(KeyCode::KeyM) if pressed => {
							music_enabled = !music_enabled;
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
				WindowEvent::MouseInput { state, button, .. } => {
					let pressed = matches!(state, ElementState::Pressed);

					match button {
						MouseButton::Left => editor.left_click(pressed),
						MouseButton::Right => editor.right_click(pressed),
						_ => {}
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					editor.mouse_move(position.x as i32, position.y as i32);
				}
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_deref_mut() {
						let frame_start = time::Instant::now();
						let frame_dt = frame_start - last_frame_start;
						last_frame_start = frame_start;

						// Gamepad support
						let pad_input = gamepads.poll();
						editor.key_left(key_left || pad_input.left);
						editor.key_right(key_right || pad_input.right);
						editor.key_up(key_up || pad_input.up);
						editor.key_down(key_down || pad_input.down);
						if !gamepad_start && pad_input.start {
							editor.toggle_play();
						}
						gamepad_start = pad_input.start;

						let edit_tool = editor.get_tool();
						if current_tool != edit_tool {
							current_tool = edit_tool;
							let cursor_icon = match edit_tool {
								Some(editor::Tool::Terrain) => CursorIcon::Crosshair,
								Some(editor::Tool::Entity) => CursorIcon::Pointer,
								Some(editor::Tool::Connection) => CursorIcon::Alias,
								None => CursorIcon::Default,
							};
							app.window.set_cursor_icon(cursor_icon);
						}

						if frame_dt >= SLOW_THRESHOLD {
							editor.think();
							tick_budget = time::Duration::ZERO;
						}
						else {
							tick_budget += frame_dt;
							while tick_budget >= FRAME_TIME {
								editor.think();
								tick_budget -= FRAME_TIME;
							}
						}

						let fx_events = editor.take_fx_events();
						for evt in fx_events {
							match evt {
								chipgame::fx::FxEvent::Sound(sound) => ap.play_sound(sound),
								chipgame::fx::FxEvent::LevelComplete => level_complete(&mut editor),
								chipgame::fx::FxEvent::GameOver => editor.toggle_play(),
								_ => {}
							}
						}

						let music = editor.get_music(music_enabled);
						ap.play_music(music);

						app.g.begin();
						let time = time_base.elapsed().as_secs_f64();
						editor.draw(&mut app.g, &app.resx, time);
						app.g.end();

						app.surface.swap_buffers(&app.context).unwrap();
					}
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = app.as_deref() {
					app.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}

fn level_complete(editor: &mut editor::EditorState) {
	let string;
	let description = if let Some(run_stats) = editor.play_stats() {
		string = format!(
			"RealTime: {:.2} sec\nTime: {} ticks\nTime: {}\nSteps: {}\nBonks: {}\n\nEmbed this run into the level file for future reference?",
			run_stats.realtime, run_stats.ticks,
			chipcore::FmtTime::new(&run_stats.ticks),
			run_stats.steps, run_stats.bonks,
		);
		&string
	}
	else {
		"Embed this run into the level file for future reference?"
	};
	let save_replay = rfd::MessageDialog::new()
		.set_title("Save replay?")
		.set_description(description)
		.set_buttons(rfd::MessageButtons::YesNo)
		.set_level(rfd::MessageLevel::Info)
		.show();
	if matches!(save_replay, rfd::MessageDialogResult::Yes) {
		editor.save_replay();
	}
	editor.toggle_play();
}
