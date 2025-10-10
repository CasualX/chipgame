#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, mem, thread, time};
use std::collections::HashMap;

mod xinput;

use chipgame::FileSystem;

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
	fn load_wav(&mut self, fx: chipcore::SoundFx, fs: &FileSystem, path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		let data = fs.read(path).expect("Failed to read sound file");
		wav.load_mem(&data).expect("Failed to load sound");
		self.sfx.insert(fx, wav);
	}
	fn load_music(&mut self, music: chipgame::data::MusicId, fs: &FileSystem,path: &str) {
		use soloud::*;
		let mut wav = Wav::default();
		let data = fs.read(path).expect("Failed to read music file");
		wav.load_mem(&data).expect("Failed to load music");
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

enum ConfigSection {
	Error,
	General,
	SoundFx,
	Music,
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
	let mut ap = AudioPlayer {
		sl,
		sfx: HashMap::new(),
		music: HashMap::new(),
		cur_music: None,
	};

	let mut section = ConfigSection::General;

	let mut tileset_texture = "tileset/MS.png";
	let mut pixel_art_bias = 0.5;
	let config_storage = fs::read_to_string("chipgame.ini");
	if let Ok(config) = &config_storage {
		for item in ini_core::Parser::new(config) {
			match item {
				ini_core::Item::Property(key, Some(value)) => {
					match section {
						ConfigSection::General => {
							match key {
								"TilesetTexture" => tileset_texture = value,
								"PixelArtBias" => {
									if let Ok(v) = value.parse::<f32>() {
										pixel_art_bias = v;
									}
								}
								_ => { /* Ignore other items */ }
							}
						}
						ConfigSection::SoundFx => {
							if let Ok(fx) = key.parse::<chipcore::SoundFx>() {
								ap.load_wav(fx, &fs, value);
							}
						}
						ConfigSection::Music => {
							if let Ok(music) = key.parse::<chipgame::data::MusicId>() {
								ap.load_music(music, &fs, value);
							}
						}
						_ => { /* Ignore other sections */ }
					}
				}
				ini_core::Item::Section(name) => {
					section = match name {
						"SoundFx" => ConfigSection::SoundFx,
						"Music" => ConfigSection::Music,
						_ => ConfigSection::Error,
					};
				}
				_ => { /* Ignore other items */ }
			}
		}
	}

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
	let tileset = load_png(&mut g, Some("scene tiles"), &fs, tileset_texture, &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, Some(&mut shade::image::gutter(32, 32))).unwrap();
	let tex_info = g.texture2d_get_info(tileset);

	let effects = load_png(&mut g, Some("effects"), &fs, "effects.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	let texdigits = load_png(&mut g, Some("digits"), &fs, "digits.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::ClampEdge,
		wrap_v: shade::TextureWrap::ClampEdge,
	}, None).unwrap();

	let menubg = load_png(&mut g, Some("menubg"), &fs, "menubg.png", &shade::image::TextureProps {
		filter_min: shade::TextureFilter::Linear,
		filter_mag: shade::TextureFilter::Linear,
		wrap_u: shade::TextureWrap::Repeat,
		wrap_v: shade::TextureWrap::Repeat,
	}, None).unwrap();

	// Create the shader
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

	let mut past_now = time::Instant::now();

	let font = {
		let font: shade::msdfgen::Font = serde_json::from_str(fs.read_to_string("font.json").unwrap().as_str()).unwrap();
		let font = Some(font);

		let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS);

		let texture = load_png(&mut g, Some("font"), &fs, "font.png", &shade::image::TextureProps {
			filter_min: shade::TextureFilter::Linear,
			filter_mag: shade::TextureFilter::Linear,
			wrap_u: shade::TextureWrap::ClampEdge,
			wrap_v: shade::TextureWrap::ClampEdge,
		}, None).unwrap();

		shade::d2::FontResource { font, shader, texture }
	};

	drop(config_storage);

	let viewport = shade::cvmath::Bounds2(
		shade::cvmath::Vec2::ZERO,
		shade::cvmath::Vec2(size.width as i32, size.height as i32),
	);
	let mut resx = chipgame::fx::Resources {
		effects,
		tileset,
		tileset_size: [tex_info.width, tex_info.height].into(),
		shader,
		pixel_art_bias,
		viewport,
		colorshader,
		uishader,
		texdigits,
		menubg,
		menubg_scale: 2.0,
		font,
	};
	let mut state = chipgame::play::PlayState::default();
	state.lvsets.load();
	state.launch(&mut g);

	let mut kbd_input = chipcore::Input::default();

	let time_base = time::Instant::now();

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

		resx.viewport.maxs = [size.width as i32, size.height as i32].into();
		let input = kbd_input | x_input;
		state.think(&input);

		g.begin();
		let time = time_base.elapsed().as_secs_f64();
		state.draw(&mut g, &resx, time);
		g.end();

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
}

fn is_pressed(state: winit::event::ElementState) -> bool {
	match state {
		winit::event::ElementState::Pressed => true,
		winit::event::ElementState::Released => false,
	}
}
