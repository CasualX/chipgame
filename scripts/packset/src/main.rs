use std::fs;

use chipty::*;

fn main() {
	let app = clap::command!("packset")
		.arg(clap::arg!(<LEVELSET_PATH> "Path to levelset directory"))
		.arg(clap::arg!(<OUTPUT_FILE> "Path to output packed levelset file"));
	let matches = app.get_matches();
	let base_path = matches.get_one::<String>("LEVELSET_PATH").expect("LEVELSET_PATH argument missing");
	let output_file = matches.get_one::<String>("OUTPUT_FILE").expect("OUTPUT_FILE argument missing");

	println!("Packing levelset {base_path} into {output_file}");

	// Load levelset at index.json
	let index_path = format!("{}/index.json", base_path);
	let mut levelset: LevelSetDto = match serde_json::from_reader(fs::File::open(index_path).expect("Failed to open levelset file")) {
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
			let level_path = format!("{}/{}", base_path, path);
			let level: LevelDto = match serde_json::from_reader(fs::File::open(level_path).expect("Failed to open level file")) {
				Ok(lv) => lv,
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
	let ref key = paks::Key::default();
	let mut paks = paks::FileEditor::create_new(output_file, key).expect("Failed to create output paks file");
	paks.create_file(b"index.json", &compressed, key).expect("Failed to add index.json to paks");
	if let Some(splash) = &levelset.splash {
		let splash_path = format!("{}/{}", base_path, splash);
		let splash_data = fs::read(&splash_path).expect("Failed to read splash image");
		paks.create_file(splash.as_bytes(), &splash_data, key).expect("Failed to add splash image to paks");
	}
	paks.finish(key).expect("Failed to finalize paks file");
}
