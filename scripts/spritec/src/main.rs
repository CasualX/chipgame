
fn main() {
	let app = clap::command!("spritec")
		.arg(clap::arg!([files] "Paths to .ini files describing the sprite").multiple_occurrences(true));
	let matches = app.get_matches();

	let files: Vec<_> = matches.values_of("files").unwrap().map(parse_sprite).collect();

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

fn parse_sprite(path: &str) -> SpriteIni {
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