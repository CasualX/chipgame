use std::{fs, path};

fn main() {
	let matches = clap::command!()
		.about("Compress and base64-encode a level JSON file for the html shell")
		.arg(
			clap::arg!(<PATH> "Path to a level JSON file")
				.value_parser(clap::value_parser!(path::PathBuf)),
		)
		.get_matches();

	let path = matches.get_one::<path::PathBuf>("PATH").expect("PATH is required");
	let level_json = match fs::read(path) {
		Ok(level_json) => level_json,
		Err(err) => {
			eprintln!("Failed to read {}: {}", path.display(), err);
			std::process::exit(1);
		}
	};

	let encoded = chipty::encode_level(&level_json);

	// let decoded = chipty::decode_level(&encoded);
	// assert_eq!(level_json, decoded);

	println!("https://casualhacks.net/chipdx/?levelc={}", encoded);
}
