use std::{fs, path};

pub struct Sprite {
	pub name: String,
	pub frames: Vec<String>,
	pub transform: chipty::SpriteTransform,
}

pub struct File {
	pub sprites: Vec<Sprite>,
}

impl File {
	pub fn load(root: &path::Path) -> File {
		let sprites = load_sprites(root);
		File { sprites }
	}
}

fn load_sprites(root: &path::Path) -> Vec<Sprite> {
	let config_path = root.join("_Sprites.ini");
	let config_text = fs::read_to_string(&config_path)
		.unwrap_or_else(|err| panic!("read sprite config {}: {}", config_path.display(), err));

	let mut sections: Vec<Sprite> = Vec::new();
	let mut current_name: Option<String> = None;
	let mut current_frames: Vec<String> = Vec::new();
	let mut current_transform = chipty::SpriteTransform::None;

	for item in ini_core::Parser::new(&config_text) {
		match item {
			ini_core::Item::Section(name) => {
				if let Some(prev_name) = current_name.take() {
					if current_frames.is_empty() {
						panic!("sprite {} missing Path entries", prev_name);
					}
					sections.push(Sprite { name: prev_name, frames: current_frames, transform: current_transform });
				}
				current_name = Some(name.to_string());
				current_transform = chipty::SpriteTransform::None;
				current_frames = Vec::new();
			}
			ini_core::Item::Property(key, Some(value)) => {
				if key == "Frame" {
					current_frames.push(value.to_string());
				}
				else if key == "Transform" {
					current_transform = match value {
						"None" => chipty::SpriteTransform::None,
						"FlipX" => chipty::SpriteTransform::FlipX,
						"FlipY" => chipty::SpriteTransform::FlipY,
						"FlipXY" => chipty::SpriteTransform::FlipXY,
						"Rotate90" => chipty::SpriteTransform::Rotate90,
						"Rotate180" => chipty::SpriteTransform::Rotate180,
						"Rotate270" => chipty::SpriteTransform::Rotate270,
						_ => panic!("unknown Transform value: {}", value),
					};
				}
			}
			_ => {}
		}
	}

	if let Some(prev_name) = current_name.take() {
		if current_frames.is_empty() {
			panic!("sprite {} missing Path entries", prev_name);
		}
		sections.push(Sprite { name: prev_name, frames: std::mem::take(&mut current_frames), transform: current_transform });
	}

	let mut sprites: Vec<Sprite> = sections;

	sprites.sort_by(|a, b| a.name.cmp(&b.name));
	sprites
}
