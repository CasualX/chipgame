#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{mem, thread, time};
use std::collections::HashMap;
use std::ffi::CString;
use std::num::NonZeroU32;

use glutin::prelude::*;
use raw_window_handle::HasRawWindowHandle;

use chipgame::FileSystem;

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

struct Config {
	tileset_texture: String,
	font_atlas: String,
	font_meta: String,
	pixel_art_bias: f32,
	sound_fx: HashMap<chipty::SoundFx, String>,
	music: HashMap<chipty::MusicId, String>,
}

fn parse_config(cfg_text: &str) -> Config {
	enum Section { Error, General, SoundFx, Music }

	let mut section = Section::General;
	let mut tileset_texture = String::from("tileset/Kayu.png");
	let mut font_atlas = String::from("font.png");
	let mut font_meta = String::from("font.json");
	let mut pixel_art_bias = 0.5f32;
	let mut sound_fx: HashMap<chipty::SoundFx, String> = HashMap::new();
	let mut music: HashMap<chipty::MusicId, String> = HashMap::new();

	for item in ini_core::Parser::new(cfg_text) {
		match item {
			ini_core::Item::Property(key, Some(value)) => match section {
				Section::General => match key {
					"FontAtlas" => font_atlas = value.to_string(),
					"FontMeta" => font_meta = value.to_string(),
					"TilesetTexture" => tileset_texture = value.to_string(),
					"PixelArtBias" => { if let Ok(v) = value.parse::<f32>() { pixel_art_bias = v; } }
					_ => {}
				},
				Section::SoundFx => {
					if let Ok(fx) = key.parse::<chipty::SoundFx>() {
						sound_fx.insert(fx, value.to_string());
					}
				}
				Section::Music => {
					if let Ok(id) = key.parse::<chipty::MusicId>() {
						music.insert(id, value.to_string());
					}
				}
				Section::Error => {}
			},
			ini_core::Item::Section(name) => {
				section = match name {
					"SoundFx" => Section::SoundFx,
					"Music" => Section::Music,
					_ => Section::Error,
				};
			}
			_ => {}
		}
	}

	Config { tileset_texture, font_atlas, font_meta, pixel_art_bias, sound_fx, music }
}

fn load_audio(ap: &mut AudioPlayer, fs: &FileSystem, config: &Config) {
	for (fx, path) in &config.sound_fx {
		ap.load_wav(*fx, fs, path);
	}
	for (id, path) in &config.music {
		ap.load_music(*id, fs, path);
	}
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
	fs: &FileSystem,
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

	// Load GL function pointers
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

	let tileset = load_png(&mut g, Some("scene tiles"), fs, config.tileset_texture.as_str(), &tex_props, Some(&mut shade::image::gutter(32, 32))).unwrap();
	let effects = load_png(&mut g, Some("effects"), fs, "effects.png", &tex_props, None).unwrap();
	let texdigits = load_png(&mut g, Some("digits"), fs, "digits.png", &tex_props, None).unwrap();
	let menubg = load_png(&mut g, Some("menubg"), fs, "menubg.png", &tex_props_repeat, None).unwrap();
	let tileset_info = g.texture2d_get_info(tileset);

	let shader = {
		let vs = fs.read_to_string("pixelart.vs.glsl").unwrap();
		let fs = fs.read_to_string("pixelart.fs.glsl").unwrap();
		g.shader_create(None, &vs, &fs)
	};
	let colorshader = {
		let vs = fs.read_to_string("color.vs.glsl").unwrap();
		let fs = fs.read_to_string("color.fs.glsl").unwrap();
		g.shader_create(None, &vs, &fs)
	};
	let uishader = {
		let vs = fs.read_to_string("ui.vs.glsl").unwrap();
		let fs = fs.read_to_string("ui.fs.glsl").unwrap();
		g.shader_create(None, &vs, &fs)
	};

	let font = {
		let font: shade::msdfgen::FontDto = serde_json::from_str(fs.read_to_string(config.font_meta.as_str()).unwrap().as_str()).unwrap();
		let font: Option<shade::msdfgen::Font> = Some(font.into());
		let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS);
		let texture = load_png(&mut g, Some("font"), fs, config.font_atlas.as_str(), &tex_props, None).unwrap();
		shade::d2::FontResource { font, shader, texture }
	};

	let viewport = shade::cvmath::Bounds2::vec(shade::cvmath::Vec2(size.width as i32, size.height as i32));
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

fn set_title(window: &winit::window::Window, state: &chipgame::play::PlayState) {
	if let Some(fx) = &state.fx {
		window.set_title(&format!("{} - Level {} - {}", state.lvsets.current().title, fx.level_number, fx.gs.field.name));
	}
	else if let Some(level_pack) = state.lvsets.collection.get(state.lvsets.selected) {
		window.set_title(&level_pack.title);
	}
	else {
		window.set_title("Play ChipGame");
	}
}

fn set_fullscreen(app: &AppStuff, fullscreen: bool) {
	// Borderless fullscreen on the current monitor; hide cursor when fullscreen
	let target = if fullscreen {
		let monitor = app.window.current_monitor();
		Some(winit::window::Fullscreen::Borderless(monitor))
	}
	else {
		None
	};
	app.window.set_fullscreen(target);
	app.window.set_cursor_visible(!fullscreen);
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

fn main() {
	let key = paks::Key::default();
	let fs = if let Ok(paks) = paks::FileReader::open("data.paks", &key) {
		FileSystem::Paks(paks, key)
	}
	else {
		FileSystem::StdFs(std::path::PathBuf::from("data"))
	};

	let xinput = xinput::XInput::new();

	let sl = soloud::Soloud::default().expect("Failed to create SoLoud");
	let mut ap = AudioPlayer { sl, sfx: HashMap::new(), music: HashMap::new(), cur_music: None };

	let config = std::fs::read_to_string("chipgame.ini").unwrap_or_default();
	let config = parse_config(config.as_str());
	load_audio(&mut ap, &fs, &config);

	let mut size = winit::dpi::PhysicalSize::new(800, 600);
	let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");

	// App state to be initialized on Event::Resumed
	let mut app: Option<AppStuff> = None;

	let mut kbd_input = chipcore::Input::default();
	let time_base = time::Instant::now();
	let mut past_now = time::Instant::now();

	let mut state = chipgame::play::PlayState::default();
	state.lvsets.load();

	use winit::event::{Event, WindowEvent};
	use winit::keyboard::{KeyCode, PhysicalKey};

	let _ = event_loop.run(move |event, elwt| {
		match event {
			Event::Resumed => {
				if app.is_none() {
					let mut built = init_app(elwt, size, &fs, &config);
					state.launch(&mut built.g);
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
				WindowEvent::KeyboardInput { event, .. } => {
					let pressed = matches!(event.state, winit::event::ElementState::Pressed);

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
				WindowEvent::RedrawRequested => {
					if let Some(app) = app.as_mut() {
						let mut x_input = chipcore::Input::default();
						xinput.get_state(&mut x_input);

						app.resx.viewport.maxs = [size.width as i32, size.height as i32].into();
						let input = kbd_input | x_input;
						state.think(&input);

						app.g.begin();
						let time = time_base.elapsed().as_secs_f64();
						state.draw(&mut app.g, &mut app.resx, time);
						app.g.end();

						for evt in &mem::replace(&mut state.events, Vec::new()) {
							match evt {
								&chipgame::play::PlayEvent::PlaySound { sound } => ap.play(sound),
								&chipgame::play::PlayEvent::PlayMusic { music } => ap.play_music(music),
								&chipgame::play::PlayEvent::Quit => elwt.exit(),
								&chipgame::play::PlayEvent::PlayLevel => set_title(&app.window, &state),
							}
						}

						app.surface.swap_buffers(&app.context).unwrap();
						let now = time::Instant::now();
						let sleep_dur = time::Duration::from_millis(24).saturating_sub(now - past_now);
						past_now = now;
						if sleep_dur > time::Duration::ZERO {
							thread::sleep(sleep_dur);
						}
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
