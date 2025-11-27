use std::{fs, path};

use chipty::*;

fn main() {
	let app = clap::command!()
		.arg(clap::arg!(<LEVELSET_PATH> "Path to levelset directory").value_parser(clap::value_parser!(path::PathBuf)))
		.arg(clap::arg!(-k --key [KEY] "Encryption key"))
		.arg(clap::arg!(<OUTPUT_FILE> "Path to output packed levelset file").value_parser(clap::value_parser!(path::PathBuf)));

	let matches = app.get_matches();
	let base_path = matches.get_one::<path::PathBuf>("LEVELSET_PATH").expect("LEVELSET_PATH argument missing");
	let output_file = matches.get_one::<path::PathBuf>("OUTPUT_FILE").expect("OUTPUT_FILE argument missing");
	let ref key = matches.get_one::<String>("key").map(|s| paks::parse_key(&s).expect("Invalid key format")).unwrap_or(paks::Key::default());

	println!("Packing levelset {} into {}", base_path.display(), output_file.display());

	// Load levelset at index.json
	let index_path = base_path.join("index.json");
	let mut levelset: LevelSetDto = match serde_json::from_reader(fs::File::open(&index_path).expect("Failed to open levelset file")) {
		Ok(ls) => ls,
		Err(e) => {
			eprintln!("Error loading levelset: {}", e);
			return;
		}
	};

	// Load all indirect levels
	for entry in &mut levelset.levels {
		if let LevelRef::Indirect(ref path) = entry {
			// Load level file
			let level_path = base_path.join(path);
			let level: LevelDto = match serde_json::from_reader(fs::File::open(&level_path).expect("Failed to open level file")) {
				Ok(level) => level,
				Err(e) => {
					eprintln!("Error loading level {}: {}", path, e);
					return;
				}
			};
			*entry = LevelRef::Direct(level);
		}
	}

	// Compress and write new index.json
	let new_index_json = serde_json::to_string(&levelset).expect("Failed to serialize levelset");
	let compressed = chipty::compress(new_index_json.as_bytes());
	println!(
		"index.json: {} bytes, compressed: {} bytes (ratio: {:.2}%)",
		new_index_json.len(),
		compressed.len(),
		(compressed.len() as f64 / new_index_json.len() as f64) * 100.0
	);

	// Create paks file
	let mut paks = paks::FileEditor::create_new(&output_file, key).expect("Failed to create output paks file");
	paks.create_file(b"index.json", &compressed, key).expect("Failed to add index.json to paks");
	if let Some(splash) = &levelset.splash {
		let splash_path = base_path.join(splash);
		let splash_data = fs::read(&splash_path).expect("Failed to read splash image");
		paks.create_file(splash.as_bytes(), &splash_data, key).expect("Failed to add splash image to paks");
	}
	paks.finish(key).expect("Failed to finalize paks file");
}
