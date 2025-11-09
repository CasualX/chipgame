use super::*;

#[derive(Default)]
pub struct Resources {
	pub effects: shade::Texture2D,
	pub tileset: shade::Texture2D,
	pub tileset_size: Vec2<i32>,
	pub shader: shade::Shader,
	pub pixel_art_bias: f32,
	pub viewport: Bounds2i,

	pub colorshader: shade::Shader,
	pub uishader: shade::Shader,
	pub menubg: shade::Texture2D,
	pub menubg_scale: f32,

	pub font: shade::d2::FontResource<Option<shade::msdfgen::Font>>,
}

#[track_caller]
fn load_png(
	g: &mut shade::Graphics,
	name: Option<&str>,
	fs: &crate::FileSystem,
	path: &str,
	props: &shade::image::TextureProps,
	transform: Option<&mut dyn FnMut(&mut Vec<u8>, &mut shade::image::ImageSize)>,
) -> Result<shade::Texture2D, shade::image::png::LoadError> {
	let data = fs.read(path).expect("Failed to read PNG file");
	shade::image::png::load_stream(g, name, &mut &data[..], props, transform)
}

impl Resources {
	pub fn load(&mut self, fs: &crate::FileSystem, config: &crate::config::Config, g: &mut shade::Graphics) {
		let resx = self;
		for (name, shader) in &config.shaders {
			let vs = fs.read_to_string(&shader.vertex_shader).expect("Failed to read shader vertex file");
			let fs = fs.read_to_string(&shader.fragment_shader).expect("Failed to read shader fragment file");
			g.shader_create(Some(name.as_str()), &vs, &fs);
		}
		for (name, texture) in &config.textures {
			let transform = match texture.transform {
				Some(crate::config::TransformType::Gutter32x32) => Some(&mut shade::image::gutter(32, 32) as _),
				_ => None,
			};
			load_png(g, Some(name.as_str()), fs, &texture.path, &texture.props, transform).expect("Failed to load texture");
		}
		let shader = g.shader_create(None, shade::gl::shaders::MTSDF_VS, shade::gl::shaders::MTSDF_FS);
		for (name, font_config) in &config.fonts {
			let font = fs.read_to_string(&font_config.meta).expect("Failed to read font meta file");
			let font: shade::msdfgen::FontDto = serde_json::from_str(&font).expect("Failed to parse font meta file");
			let font: Option<shade::msdfgen::Font> = Some(font.into());
			let texture = load_png(g, Some(name.as_str()), fs, &font_config.atlas, &shade::image::TextureProps {
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::ClampEdge,
				wrap_v: shade::TextureWrap::ClampEdge,
			}, None).expect("Failed to load font atlas");
			let font = shade::d2::FontResource { font, shader, texture };
			resx.font = font;
		}

		resx.effects = g.texture2d_find("Effects");
		resx.tileset = g.texture2d_find("Tileset");
		resx.tileset_size = {
			let info = g.texture2d_get_info(resx.tileset);
			[info.width, info.height].into()
		};
		resx.shader = g.shader_find("PixelArt");
		resx.pixel_art_bias = config.pixel_art_bias;
		resx.colorshader = g.shader_find("Color");
		resx.uishader = g.shader_find("UI");
		resx.menubg = g.texture2d_find("MenuBG");
		resx.menubg_scale = 2.0;
	}
}
