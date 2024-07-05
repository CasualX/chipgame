use std::{fs, thread, time};

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

fn main() {
	let Some(file_path) = std::env::args_os().nth(1) else {
		panic!("Usage: cargo run --example edit <level>");
	};

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
	if let Err(_) = g.shader_compile(shader, include_str!("../../../data/standard.vs.glsl"), include_str!("../../../data/standard.fs.glsl")) {
		panic!("Failed to compile shader: {}", g.shader_compile_log(shader).unwrap());
	}

	let mut past_now = time::Instant::now();

	let mut editor = chipgame::editor::EditorGame::default();
	let mut input = chipgame::editor::EditorInput::default();
	editor.load_level(&fs::read_to_string(&file_path).unwrap());

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
						Some(winit::event::VirtualKeyCode::Left) => input.left = is_pressed(state),
						Some(winit::event::VirtualKeyCode::Right) => input.right = is_pressed(state),
						Some(winit::event::VirtualKeyCode::Up) => input.up = is_pressed(state),
						Some(winit::event::VirtualKeyCode::Down) => input.down = is_pressed(state),
						Some(winit::event::VirtualKeyCode::A) if is_pressed(state) => input.chr = Some('A'),
						Some(winit::event::VirtualKeyCode::B) if is_pressed(state) => input.chr = Some('B'),
						Some(winit::event::VirtualKeyCode::C) if is_pressed(state) => input.chr = Some('C'),
						Some(winit::event::VirtualKeyCode::D) if is_pressed(state) => input.chr = Some('D'),
						Some(winit::event::VirtualKeyCode::E) if is_pressed(state) => input.chr = Some('E'),
						Some(winit::event::VirtualKeyCode::F) if is_pressed(state) => input.chr = Some('F'),
						Some(winit::event::VirtualKeyCode::G) if is_pressed(state) => input.chr = Some('G'),
						Some(winit::event::VirtualKeyCode::H) if is_pressed(state) => input.chr = Some('H'),
						Some(winit::event::VirtualKeyCode::I) if is_pressed(state) => input.chr = Some('I'),
						Some(winit::event::VirtualKeyCode::J) if is_pressed(state) => input.chr = Some('J'),
						Some(winit::event::VirtualKeyCode::K) if is_pressed(state) => input.chr = Some('K'),
						Some(winit::event::VirtualKeyCode::L) if is_pressed(state) => input.chr = Some('L'),
						Some(winit::event::VirtualKeyCode::M) if is_pressed(state) => input.chr = Some('M'),
						Some(winit::event::VirtualKeyCode::N) if is_pressed(state) => input.chr = Some('N'),
						Some(winit::event::VirtualKeyCode::O) if is_pressed(state) => input.chr = Some('O'),
						Some(winit::event::VirtualKeyCode::P) if is_pressed(state) => input.chr = Some('P'),
						Some(winit::event::VirtualKeyCode::Q) if is_pressed(state) => input.chr = Some('Q'),
						Some(winit::event::VirtualKeyCode::R) if is_pressed(state) => input.chr = Some('R'),
						Some(winit::event::VirtualKeyCode::S) if is_pressed(state) => input.chr = Some('S'),
						Some(winit::event::VirtualKeyCode::T) if is_pressed(state) => input.chr = Some('T'),
						Some(winit::event::VirtualKeyCode::U) if is_pressed(state) => input.chr = Some('U'),
						Some(winit::event::VirtualKeyCode::V) if is_pressed(state) => input.chr = Some('V'),
						Some(winit::event::VirtualKeyCode::W) if is_pressed(state) => input.chr = Some('W'),
						Some(winit::event::VirtualKeyCode::X) if is_pressed(state) => input.chr = Some('X'),
						Some(winit::event::VirtualKeyCode::Y) if is_pressed(state) => input.chr = Some('Y'),
						Some(winit::event::VirtualKeyCode::Z) if is_pressed(state) => input.chr = Some('Z'),
						Some(winit::event::VirtualKeyCode::F5) if is_pressed(state) => {
							let s = editor.save_level();
							fs::write(&file_path, s).unwrap();
						}
						_ => (),
					}
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::MouseInput { state, button, .. }, .. } => {
					match button {
						winit::event::MouseButton::Left => input.left_click = is_pressed(state),
						winit::event::MouseButton::Right => input.right_click = is_pressed(state),
						_ => (),
					}
				}
				winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CursorMoved { position, .. }, .. } => {
					input.mouse.x = position.x as i32;
					input.mouse.y = position.y as i32;
				}
				winit::event::Event::MainEventsCleared => {
					*control_flow = winit::event_loop::ControlFlow::Exit;
				}
				_ => (),
			}
		});

		input.screen_size.x = size.width as i32;
		input.screen_size.y = size.height as i32;

		editor.init(chipgame::visual::Resources {
			tileset,
			tileset_size: [tex_info.width, tex_info.height].into(),
			shader,
			screen_size: [size.width as i32, size.height as i32].into(),
		});
		editor.render(&mut g, &input);

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
