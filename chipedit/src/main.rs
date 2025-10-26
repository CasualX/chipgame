#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, thread, time};
use std::ffi::CString;
use std::num::NonZeroU32;
use std::path::PathBuf;

use glutin::prelude::*;
use raw_window_handle::HasRawWindowHandle;

use chipgame::editor;
use chipgame::FileSystem;

#[cfg(windows)]
fn window_builder(size: winit::dpi::PhysicalSize<u32>) -> winit::window::WindowBuilder {
	use winit::platform::windows::WindowBuilderExtWindows;
	winit::window::WindowBuilder::new()
		.with_title("ChipEdit")
		.with_inner_size(size)
		.with_drag_and_drop(false)
}
#[cfg(not(windows))]
fn window_builder(size: winit::dpi::PhysicalSize<u32>) -> winit::window::WindowBuilder {
	winit::window::WindowBuilder::new()
		.with_title("ChipEdit")
		.with_inner_size(size)
}

#[track_caller]
fn load_png(
	g: &mut shade::Graphics,
	name: Option<&str>,
	fs: &FileSystem,
	path: &str,
	props: &shade::image::TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut shade::image::ImageSize)>,
) -> Result<shade::Texture2D, shade::image::png::LoadError> {
	let data = fs.read(path).expect("Failed to read PNG file");
	shade::image::png::load_stream(g, name, &mut &data[..], props, transform)
}

struct Config {
	tileset_texture: String,
	font_atlas: String,
	font_meta: String,
	pixel_art_bias: f32,
}

fn parse_config(cfg_text: &str) -> Config {
	let mut tileset_texture = "tileset/Kayu.png".to_string();
	let mut font_atlas = "font.png".to_string();
	let mut font_meta = "font.json".to_string();
	let mut pixel_art_bias = 0.5f32;

	for item in ini_core::Parser::new(cfg_text) {
		match item {
			ini_core::Item::Property(key, Some(value)) => match key {
				"TilesetTexture" => tileset_texture = value.to_string(),
				"FontAtlas" => font_atlas = value.to_string(),
				"FontMeta" => font_meta = value.to_string(),
				"PixelArtBias" => if let Ok(v) = value.parse::<f32>() { pixel_art_bias = v; },
				_ => {}
			},
			_ => {}
		}
	}

	Config { tileset_texture, font_atlas, font_meta, pixel_art_bias }
}

struct AppStuff {
	window: winit::window::Window,
	surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
	context: glutin::context::PossiblyCurrentContext,
	g: shade::gl::GlGraphics,
	resx: chipgame::fx::Resources,
}

fn init_app(
	elwt: &winit::event_loop::EventLoopWindowTarget<()>,
	size: winit::dpi::PhysicalSize<u32>,
	vfs: &FileSystem,
	config: &Config,
) -> AppStuff {
	use glutin::config::ConfigTemplateBuilder;
	use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
	use glutin::display::GetGlDisplay;
	use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};

	let template = ConfigTemplateBuilder::new()
		.with_alpha_size(8)
		.with_multisampling(4);

	let (window, gl_config) = glutin_winit::DisplayBuilder::new()
		.with_window_builder(Some(window_builder(size)))
		.build(elwt, template, |configs| configs.max_by_key(|c| c.num_samples()).unwrap().clone())
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
	let tex_props = shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	};
	let tex_props_repeat = shade::image::TextureProps {
		wrap_u: shade::TextureWrap::Repeat,
		wrap_v: shade::TextureWrap::Repeat,
		..tex_props
	};

	let tileset = load_png(&mut g, Some("scene tiles"), vfs, &config.tileset_texture, &tex_props, Some(&mut shade::image::gutter(32, 32))).unwrap();
	let effects = load_png(&mut g, Some("effects"), vfs, "effects.png", &tex_props, None).unwrap();
	let texdigits = load_png(&mut g, Some("digits"), vfs, "digits.png", &tex_props, None).unwrap();
	let menubg = load_png(&mut g, Some("menubg"), vfs, "menubg.png", &tex_props_repeat, None).unwrap();
	let tileset_info = g.texture2d_get_info(tileset);

	let shader = {
		let vs = vfs.read_to_string("pixelart.vs.glsl").unwrap();
		let fs_src = vfs.read_to_string("pixelart.fs.glsl").unwrap();
		g.shader_create(None, &vs, &fs_src)
	};
	let colorshader = {
		let vs = vfs.read_to_string("color.vs.glsl").unwrap();
		let fs_src = vfs.read_to_string("color.fs.glsl").unwrap();
		g.shader_create(None, &vs, &fs_src)
	};
	let uishader = {
		let vs = vfs.read_to_string("ui.vs.glsl").unwrap();
		let fs_src = vfs.read_to_string("ui.fs.glsl").unwrap();
		g.shader_create(None, &vs, &fs_src)
	};

	let font = {
		let dto: shade::msdfgen::FontDto = serde_json::from_str(vfs.read_to_string(&config.font_meta).unwrap().as_str()).unwrap();
		let font: Option<shade::msdfgen::Font> = Some(dto.into());
		let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS);
		let texture = load_png(&mut g, Some("font"), vfs, &config.font_atlas, &tex_props, None).unwrap();
		shade::d2::FontResource { font, shader, texture }
	};

	let viewport = cvmath::Bounds2::vec(cvmath::Vec2(size.width as i32, size.height as i32));
	let resx = chipgame::fx::Resources {
		effects,
		tileset,
		tileset_size: [tileset_info.width, tileset_info.height].into(),
		shader,
		pixel_art_bias: config.pixel_art_bias,
		viewport,
		colorshader,
		uishader,
		texdigits,
		menubg,
		menubg_scale: 2.0,
		font,
	};

	AppStuff { window, surface, context, g, resx }
}

fn set_fullscreen(app: &AppStuff, fullscreen: bool) {
	// Borderless fullscreen on the current monitor
	let target = if fullscreen {
		let monitor = app.window.current_monitor();
		Some(winit::window::Fullscreen::Borderless(monitor))
	}
	else {
		None
	};
	app.window.set_fullscreen(target);
}

fn main() {
	let config = fs::read_to_string("chipgame.ini").unwrap_or_default();
	let config = parse_config(&config);

	// VFS
	let key = paks::Key::default();
	let vfs = if let Ok(paks) = paks::FileReader::open("data.paks", &key) {
		FileSystem::Paks(paks, key)
	}
	else {
		FileSystem::StdFs(PathBuf::from("data"))
	};

	// CLI: optional level path
	let app = clap::command!("play").arg(clap::arg!([level] "Level file to open"));
	let matches = app.get_matches();
	let mut file_path = matches.value_of("level").map(PathBuf::from);

	let mut size = winit::dpi::PhysicalSize::new(800, 600);
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	// App state
	let mut app: Option<AppStuff> = None;
	let mut editor = editor::EditorState::default();
	editor.init();

	let mut past_now = time::Instant::now();

	let _ = event_loop.run(move |event, elwt| {
		use winit::event::{Event, WindowEvent};
		use winit::keyboard::{KeyCode, PhysicalKey};

		match event {
			Event::Resumed => {
				if app.is_none() {
					let built = init_app(elwt, size, &vfs, &config);

					// Window title and initial level
					built.window.set_title("ChipEdit - (unsaved)");
					if let Some(fp) = file_path.as_ref() {
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
					size = new_size;
					if let Some(app) = app.as_ref() {
						let w = NonZeroU32::new(size.width.max(1)).unwrap();
						let h = NonZeroU32::new(size.height.max(1)).unwrap();
						app.surface.resize(&app.context, w, h);
					}
				}
				WindowEvent::CloseRequested => elwt.exit(),
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_mut() {
						app.resx.viewport.maxs = [size.width as i32, size.height as i32].into();
						editor.set_screen_size(size.width as i32, size.height as i32);

						app.g.begin();
						editor.draw(&mut app.g, &app.resx);
						app.g.end();

						app.surface.swap_buffers(&app.context).unwrap();
						let now = time::Instant::now();
						let sleep_dur = time::Duration::from_millis(24).saturating_sub(now - past_now);
						past_now = now;
						thread::sleep(sleep_dur);
					}
				}
				WindowEvent::KeyboardInput { event, .. } => {
					let pressed = matches!(event.state, winit::event::ElementState::Pressed);

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
										if let Some(app) = app.as_ref() {
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
									if let Some(app) = app.as_ref() {
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
							if let Some(app) = app.as_mut() {
								let want_fullscreen = app.window.fullscreen().is_none();
								set_fullscreen(app, want_fullscreen);
							}
						}
						PhysicalKey::Code(KeyCode::Escape) if pressed => {
							if let Some(app) = app.as_mut() {
								set_fullscreen(app, false);
							}
						}
						_ => {}
					}
				}
				WindowEvent::MouseInput { state, button, .. } => {
					match button {
						winit::event::MouseButton::Left => editor.left_click(matches!(state, winit::event::ElementState::Pressed)),
						winit::event::MouseButton::Right => editor.right_click(matches!(state, winit::event::ElementState::Pressed)),
						_ => {}
					}
				}
				WindowEvent::CursorMoved { position, .. } => {
					editor.mouse_move(position.x as i32, position.y as i32);
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
