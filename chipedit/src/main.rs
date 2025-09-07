#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{fs, thread, time};

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

fn load_png(
	g: &mut shade::Graphics,
	name: Option<&str>,
	fs: &FileSystem,
	path: &str,
	props: &shade::image::TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut shade::image::ImageSize)>,
) -> Result<shade::Texture2D, shade::image::png::LoadError> {
	let data = fs.read(path).expect("Failed to read PNG file");
	shade::image::png::load(g, name, &mut &data[..], props, transform)
}

fn main() {
	let fs = if let Ok(paks) = paks::FileReader::open("data.paks", &paks::Key::default()) {
		FileSystem::Paks(paks)
	}
	else {
		FileSystem::StdFs(std::path::PathBuf::from("data"))
	};

	let app = clap::command!("play")
		.arg(clap::arg!(<level> "Level file to play"));
	let matches = app.get_matches();
	let file_path = matches.value_of("level").unwrap();

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
	let tileset = load_png(&mut g, Some("scene tiles"), &fs, "Color_Tileset.png", &shade::image::TextureProps {
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

	// Create the shader
	let shader = {
		let vs = fs.read_to_string("standard.vs.glsl").unwrap();
		let fs = fs.read_to_string("standard.fs.glsl").unwrap();
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

	let viewport = cvmath::Bounds2(cvmath::Vec2::ZERO, cvmath::Vec2(size.width as i32, size.height as i32));
	let mut resx = chipgame::fx::Resources {
		effects,
		tileset,
		tileset_size: [tex_info.width, tex_info.height].into(),
		shader,
		viewport,
		colorshader,
		uishader,
		texdigits,
		font,
	};

	let mut editor = editor::EditorState::default();
	editor.init();
	editor.load_level(&fs::read_to_string(&file_path).unwrap());

	context.window().set_title(&format!("ChipEdit - {}", file_path));

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

			editor.set_screen_size(size.width as i32, size.height as i32);

			match event {
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::Resized(new_size), .. } => {
					size = new_size;
					context.resize(new_size);
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
					quit = true;
				}
				winit::event::Event::WindowEvent {
					event: winit::event::WindowEvent::KeyboardInput {
						input: winit::event::KeyboardInput {
							virtual_keycode,
							state,
							..
						},
						..
					},
					..
				} => {
					match virtual_keycode {
						Some(winit::event::VirtualKeyCode::Left) => editor.key_left(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Right) => editor.key_right(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Up) => editor.key_up(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Down) => editor.key_down(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Delete) => editor.delete(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::F5) if is_pressed(state) => {
							let s = editor.save_level();
							fs::write(&file_path, s).unwrap();
						}
						Some(winit::event::VirtualKeyCode::T) => editor.tool_terrain(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::E) => editor.tool_entity(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::C) => editor.tool_connection(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Numpad8) => editor.crop_top(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Numpad2) => editor.crop_bottom(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Numpad4) => editor.crop_left(is_pressed(state)),
						Some(winit::event::VirtualKeyCode::Numpad6) => editor.crop_right(is_pressed(state)),
						_ => (),
					}
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button, .. }, .. } => {
					match button {
						winit::event::MouseButton::Left => editor.left_click(is_pressed(state)),
						winit::event::MouseButton::Right => editor.right_click(is_pressed(state)),
						_ => (),
					}
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
					editor.mouse_move(position.x as i32, position.y as i32);
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		resx.viewport.maxs = [size.width as i32, size.height as i32].into();

		g.begin();
		editor.draw(&mut g, &resx);
		g.end();

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
