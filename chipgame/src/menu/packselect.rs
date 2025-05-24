use super::*;

#[derive(Default)]
pub struct LevelPackSelectMenu {
	pub selected: usize,
	pub items: Vec<String>,
	pub splash: Vec<Option<shade::image::AnimatedImage>>,
	pub ntime: i32,
}

impl LevelPackSelectMenu {
	pub fn think(&mut self, input: &Input, events: &mut Vec<MenuEvent>) {
		if input.up.is_pressed() {
			if self.selected > 0 {
				self.selected = self.selected - 1;
				events.push(MenuEvent::CursorMove);
			}
		}
		if input.down.is_pressed() {
			if self.selected + 1 < self.items.len() {
				self.selected = self.selected + 1;
				events.push(MenuEvent::CursorMove);
			}
		}
		if input.a.is_pressed() || input.start.is_pressed() {
			events.push(MenuEvent::LevelPackSelect { index: self.selected });
			events.push(MenuEvent::CursorSelect);
		}
		if input.b.is_pressed() {
			events.push(MenuEvent::CloseMenu);
		}
	}
	pub fn draw(&mut self, g: &mut shade::Graphics, resx: &Resources) {
		self.ntime += 1;

		if let Some(Some(splash)) = self.splash.get(self.selected) {
			let ss = resx.screen_size;
			let mut cv = shade::d2::CommandBuffer::<UiVertex, UiUniform>::new();
			cv.shader = resx.uishader;
			cv.blend_mode = shade::BlendMode::Alpha;
			cv.viewport = Bounds::vec(ss);

			let transform = foo(Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32), Rect::c(-1.0, 1.0, 1.0, -1.0));

			let time = self.ntime as f32 / 60.0;
			let texture = splash.get_frame(time);

			cv.push_uniform(UiUniform {
				transform,
				texture,
				..Default::default()
			});

			let color = [128, 128, 128, 255];
			let stamp = shade::d2::Stamp {
				bottom_left: UiVertex { pos: Vec2f(0.0, 0.0), uv: Vec2f(0.0, 1.0), color },
				bottom_right: UiVertex { pos: Vec2f(ss.x as f32, 0.0), uv: Vec2f(1.0, 1.0), color },
				top_left: UiVertex { pos: Vec2f(0.0, ss.y as f32), uv: Vec2f(0.0, 0.0), color },
				top_right: UiVertex { pos: Vec2f(ss.x as f32, ss.y as f32), uv: Vec2f(1.0, 0.0), color },
			};
			let rc = cvmath::Rect::c(0.0, 0.0, ss.x as f32, ss.y as f32);
			let height = splash.height as f32 * (ss.x as f32 / splash.width as f32);
			let [_, rc, _] = draw::flexv(rc, None, layout::Justify::Center, &[layout::Unit::Fr(1.0), layout::Unit::Abs(height), layout::Unit::Fr(1.0)]);
			// let [_, rc, _] = draw::flexh(rc, None, layout::Justify::Center, &[layout::Unit::Fr(1.0), layout::Unit::Abs(splash.width as f32), layout::Unit::Fr(1.0)]);
			cv.stamp_rect(&stamp, &rc);

			cv.draw(g, shade::Surface::BACK_BUFFER).unwrap();
		}


		let mut buf = shade::d2::TextBuffer::new();
		buf.shader = resx.font.shader;
		buf.blend_mode = shade::BlendMode::Alpha;
		buf.viewport = cvmath::Rect::vec(resx.screen_size);

		let rect = Rect::vec(resx.screen_size.cast::<f32>());
		let transform = foo(rect, Rect::c(-1.0, 1.0, 1.0, -1.0));

		buf.push_uniform(shade::d2::TextUniform {
			transform,
			texture: resx.font.texture,
			..Default::default()
		});

		let [top, bottom, _] = draw::flexv(rect, None, layout::Justify::Center, &[layout::Unit::Fr(1.0), layout::Unit::Fr(3.0), layout::Unit::Fr(1.0)]);

		{
			let size = resx.screen_size.y as f32 * FONT_SIZE;

			let scribe = shade::d2::Scribe {
				font_size: size,
				line_height: size * (5.0 / 4.0),
				color: cvmath::Vec4(255, 255, 255, 255),
				..Default::default()
			};

			buf.text_box(&resx.font, &scribe, &top, shade::d2::BoxAlign::MiddleCenter, "Choose a Level Pack");
		}

		let items = self.items.iter().map(|s| s as &dyn fmt::Display).collect::<Vec<_>>();

		draw::DrawMenuItems {
			items_text: &items,
			selected_index: self.selected,
		}.draw(&mut buf, &bottom, resx);

		buf.draw(g, shade::Surface::BACK_BUFFER).unwrap();
	}
}
