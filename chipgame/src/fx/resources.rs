use super::*;

#[cfg(target_arch = "wasm32")]
use shade::webgl::shaders as shaders;

#[cfg(not(target_arch = "wasm32"))]
use shade::gl::shaders as shaders;

#[derive(Default)]
pub struct Resources {
	pub textures: HashMap<String, shade::Texture2D>,
	pub shaders: HashMap<String, shade::ShaderProgram>,

	pub effects: shade::Texture2D,
	pub spritesheet_texture: shade::Texture2D,
	pub spritesheet_meta: chipty::SpriteSheet<chipty::SpriteId>,

	pub shader: shade::ShaderProgram,
	pub shader_shadowmap: shade::ShaderProgram,
	pub pixel_art_bias: f32,
	pub viewport: Bounds2i,

	pub colorshader: shade::ShaderProgram,
	pub uishader: shade::ShaderProgram,
	pub menubg: shade::Texture2D,
	pub menubg_scale: f32,

	pub font: shade::d2::FontResource<Option<shade::msdfgen::Font>>,
}

#[track_caller]
fn load_png(
	resx: &mut Resources,
	g: &mut shade::Graphics,
	name: Option<&str>,
	fs: &crate::FileSystem,
	path: &str,
	props: &shade::TextureProps,
) -> Result<shade::Texture2D, shade::image::LoadImageError> {
	let data = fs.read(path).expect("Failed to read PNG file");
	let image = shade::image::DecodedImage::load_memory_png(data.as_slice())?;
	let tex = g.image(&(&image, props));
	if let Some(name) = name {
		resx.textures.insert(name.to_string(), tex);
	}
	Ok(tex)
}

impl Resources {
	pub fn load(&mut self, fs: &crate::FileSystem, config: &crate::config::Config, g: &mut shade::Graphics) {
		let resx = self;
		for (name, shader) in &config.shaders {
			let vs = fs.read_to_string(&shader.vertex_shader).expect("Failed to read shader vertex file");
			let fs = fs.read_to_string(&shader.fragment_shader).expect("Failed to read shader fragment file");
			let shader = g.shader_compile(&vs, &fs);
			resx.shaders.insert(name.to_string(), shader);
		}
		for (name, texture) in &config.textures {
			load_png(resx, g, Some(name.as_str()), fs, &texture.path, &texture.props).expect("Failed to load texture");
		}
		let shader = g.shader_compile(shaders::MTSDF_VS, shaders::MTSDF_FS);
		for (name, font_config) in &config.fonts {
			let font = fs.read_to_string(&font_config.meta).expect("Failed to read font meta file");
			let font: shade::msdfgen::FontDto = serde_json::from_str(&font).expect("Failed to parse font meta file");
			let font: Option<shade::msdfgen::Font> = Some(font.into());
			let data = fs.read(&font_config.atlas).expect("Failed to read font atlas file");
			let image = shade::image::DecodedImage::load_memory_png(data.as_slice()).expect("Failed to decode font atlas PNG");
			let image = image.to_rgba().map_colors(|[r, g, b, a]| shade::color::Rgba8 { r, g, b, a });
			let props = shade::TextureProps {
				mip_levels: 1,
				usage: shade::TextureUsage::TEXTURE,
				filter_min: shade::TextureFilter::Linear,
				filter_mag: shade::TextureFilter::Linear,
				wrap_u: shade::TextureWrap::Edge,
				wrap_v: shade::TextureWrap::Edge,
				..Default::default()
			};
			let texture = g.image(&(&image, &props));
			resx.textures.insert(name.to_string(), texture);
			// let texture = load_png(g, Some(name.as_str()), fs, &font_config.atlas, &).expect("Failed to load font atlas");
			let font = shade::d2::FontResource { font, shader, texture };
			resx.font = font;
		}

		resx.effects = resx.textures.get("Effects").unwrap().clone();
		resx.spritesheet_texture = resx.textures.get("SpriteSheet").unwrap().clone();
		let spritesheet_meta = fs.read_to_string("spritesheet.json").expect("Failed to read spritesheet metadata");
		resx.spritesheet_meta = serde_json::from_str(&spritesheet_meta).expect("Failed to parse spritesheet metadata");

		resx.shader = resx.shaders.get("PixelArt").unwrap().clone();
		resx.shader_shadowmap = resx.shaders.get("PixelArtShadowMap").unwrap().clone();
		resx.pixel_art_bias = config.pixel_art_bias;
		resx.colorshader = resx.shaders.get("Color").unwrap().clone();
		resx.uishader = resx.shaders.get("UI").unwrap().clone();
		resx.menubg = resx.textures.get("MenuBG").unwrap().clone();
		resx.menubg_scale = 2.0;
	}
}
