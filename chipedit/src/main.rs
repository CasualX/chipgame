#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, thread, time};
use std::ffi::CString;
use std::num::NonZeroU32;
use std::path::PathBuf;

use glutin::prelude::*;

use chipgame::editor;
use chipgame::FileSystem;

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

		// Load GL functions for shaders/textures
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

fn main() {
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
		FileSystem::StdFs(PathBuf::from("data"))
	};

	// CLI: optional level path
	let app = clap::command!("play").arg(clap::arg!([level] "Level file to open"));
	let matches = app.get_matches();
	let mut file_path = matches.value_of("level").map(PathBuf::from);

	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	// App state
	let mut app: Option<AppStuff> = None;
	let mut editor = editor::EditorState::default();
	editor.init();
	let mut current_tool = None;

	let mut past_now = time::Instant::now();

	let _ = event_loop.run(move |event, elwt| {
		use winit::event::{ElementState, Event, MouseButton, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};

		match event {
			Event::Resumed => {
				if app.is_none() {
					let built = AppStuff::new(elwt, &fs, &config);

					// Window title and initial level
					built.window.set_title("ChipEdit - (unsaved)");
					if let Some(fp) = &file_path {
						if let Ok(contents) = fs::read_to_string(fp) {
							editor.load_level(&contents);
							built.window.set_title(&format!("ChipEdit - {}", fp.display()));
						}
					}
					else {
						editor.load_level(include_str!("template.json"));
					}

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
						app.resx.viewport.maxs = [app.size.width as i32, app.size.height as i32].into();
					}
					editor.set_screen_size(new_size.width as i32, new_size.height as i32);
				}
				WindowEvent::CloseRequested => elwt.exit(),
				WindowEvent::KeyboardInput { event, .. } => {
					let pressed = matches!(event.state, ElementState::Pressed);

					match event.physical_key {
						PhysicalKey::Code(KeyCode::ArrowLeft) => editor.key_left(pressed),
						PhysicalKey::Code(KeyCode::ArrowRight) => editor.key_right(pressed),
						PhysicalKey::Code(KeyCode::ArrowUp) => editor.key_up(pressed),
						PhysicalKey::Code(KeyCode::ArrowDown) => editor.key_down(pressed),
						PhysicalKey::Code(KeyCode::Delete) => editor.delete(pressed),
						PhysicalKey::Code(KeyCode::F2) if pressed => {
							// Open file dialog to load a level
							let mut dialog = rfd::FileDialog::new()
								.add_filter("Level", &["json"])
								.set_title("Open Level");
							if let Some(ref existing) = file_path {
								if let Some(parent) = existing.parent() {
									dialog = dialog.set_directory(parent);
								}
							}
							if let Some(path) = dialog.pick_file() {
								match fs::read_to_string(&path) {
									Ok(contents) => {
										editor.load_level(&contents);
										file_path = Some(path.clone());
										if let Some(app) = &app {
											app.window.set_title(&format!("ChipEdit - {}", file_path.as_ref().unwrap().display()));
										}
									}
									Err(e) => eprintln!("Failed to open level: {e}"),
								}
							}
						}
						PhysicalKey::Code(KeyCode::F5) if pressed => {
							let contents = editor.save_level();
							let mut dialog = rfd::FileDialog::new()
								.add_filter("Level", &["json"])
								.set_title("Save Level");
							if let Some(ref existing) = file_path {
								if let Some(parent) = existing.parent() {
									dialog = dialog.set_directory(parent);
								}
								if let Some(name) = existing.file_name().and_then(|s| s.to_str()) {
									dialog = dialog.set_file_name(name);
								}
							}
							if let Some(path) = dialog.save_file() {
								if let Err(e) = fs::write(&path, contents) {
									eprintln!("Failed to save level: {e}");
								}
								else {
									file_path = Some(path.clone());
									if let Some(app) = &app {
										app.window.set_title(&format!("ChipEdit - {}", file_path.as_ref().unwrap().display()));
									}
								}
							}
						}
						PhysicalKey::Code(KeyCode::KeyT) => editor.tool_terrain(pressed),
						PhysicalKey::Code(KeyCode::KeyE) => editor.tool_entity(pressed),
						PhysicalKey::Code(KeyCode::KeyC) => editor.tool_connection(pressed),
						PhysicalKey::Code(KeyCode::Numpad8) => editor.crop_top(pressed),
						PhysicalKey::Code(KeyCode::Numpad2) => editor.crop_bottom(pressed),
						PhysicalKey::Code(KeyCode::Numpad4) => editor.crop_left(pressed),
						PhysicalKey::Code(KeyCode::Numpad6) => editor.crop_right(pressed),
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
					if let Some(app) = &mut app {
						if current_tool != Some(editor.tool) {
							current_tool = Some(editor.tool);
							let cursor_icon = match editor.tool {
								editor::Tool::Terrain => winit::window::CursorIcon::Crosshair,
								editor::Tool::Entity => winit::window::CursorIcon::Pointer,
								editor::Tool::Connection => winit::window::CursorIcon::Alias,
							};
							app.window.set_cursor_icon(cursor_icon);
						}

						app.g.begin();
						editor.draw(&mut app.g, &app.resx);
						app.g.end();

						app.surface.swap_buffers(&app.context).unwrap();
					}

					let now = time::Instant::now();
					let sleep_dur = time::Duration::from_millis(24).saturating_sub(now - past_now);
					past_now = now;
					thread::sleep(sleep_dur);
				}
				_ => {}
			},
			Event::AboutToWait => {
				if let Some(app) = &app {
					app.window.request_redraw();
				}
			}
			_ => {}
		}
	});
}
