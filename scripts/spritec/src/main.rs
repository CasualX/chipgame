use std::path;

fn main() {
	let matches = clap::command!()
		.arg(clap::arg!([files] "Paths to .ini files describing the sprite").num_args(1..).value_parser(clap::value_parser!(path::PathBuf)))
		.get_matches();

	let files: Vec<_> = matches
		.get_many::<path::PathBuf>("files")
		.expect("Expected at least one sprite file")
		.map(|path| parse_sprite(path))
		.collect();

	eprintln!("{:#?}", files);
}

#[derive(Clone, Debug, Default)]
struct FrameIni {
	path: String,
	gutter: i32,
}

#[derive(Clone, Debug, Default)]
struct SpriteIni {
	id: String,
	frames: Vec<FrameIni>,
}

fn parse_sprite(path: &path::Path) -> SpriteIni {
	use ini_core as minip;

	let contents = std::fs::read_to_string(path).unwrap();

	let mut sprite = SpriteIni::default();

	let parser = minip::Parser::new(&contents);
	let mut is_frame = false;

	for line in parser {
		match line {
			minip::Item::Property("Id", Some(value)) if !is_frame => {
				sprite.id = value.to_string();
			}
			minip::Item::Section("[Frames]") => {
				is_frame = true;
				sprite.frames.push(FrameIni::default());
			}
			minip::Item::Property("Path", Some(value)) if is_frame => {
				sprite.frames.last_mut().unwrap().path = value.to_string();
			}
			minip::Item::Property("Gutter", Some(value)) if is_frame => {
				sprite.frames.last_mut().unwrap().gutter = value.parse().unwrap();
			}
			_ => (),
		}
	}

	sprite
}