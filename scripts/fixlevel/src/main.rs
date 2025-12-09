use std::{fs, path};

fn main() {
	let matches = clap::command!()
		.about("Backfill author metadata in level JSON files using a reference DAT file")
		.arg(clap::arg!(<DAT_FILE> "Path to the source levelset .dat").value_parser(clap::value_parser!(path::PathBuf)))
		.arg(clap::arg!(<LEVELSET_DIR> "Path to the unpacked levelset directory").value_parser(clap::value_parser!(path::PathBuf)))
		.get_matches();

	let dat_path = matches.get_one::<path::PathBuf>("DAT_FILE").unwrap().clone();
	let levelset_dir = matches.get_one::<path::PathBuf>("LEVELSET_DIR").unwrap().clone();

	let opts = chipdat::Options { encoding: chipdat::Encoding::Windows1252 };
	let dat = chipdat::read(&dat_path, &opts).expect("Failed to parse DAT file");

	let index_path = levelset_dir.join("index.json");
	let index_data = fs::read_to_string(&index_path).expect("Failed to read levelset index");
	let levelset: chipty::LevelSetDto = serde_json::from_str(&index_data).expect("Invalid levelset index");

	let mut level_files = Vec::new();
	for (i, level_ref) in levelset.levels.iter().enumerate() {
		match level_ref {
			chipty::LevelRef::Indirect(file) => level_files.push(levelset_dir.join(file)),
			chipty::LevelRef::Direct(_) => panic!("Level {} stored inline; run on an indirect levelset", i + 1),
		}
	}

	if dat.levels.len() != level_files.len() {
		panic!(
			"Level count mismatch ({} in DAT, {} on disk)",
			dat.levels.len(),
			level_files.len()
		);
	}

	let mut updated = 0usize;
	for (idx, (level, level_path)) in dat.levels.iter().zip(level_files.iter()).enumerate() {
		let author = level
			.metadata
			.author
			.as_deref()
			.map(|s| s.trim())
			.filter(|s| !s.is_empty())
			.map(|s| s.to_string());

		let contents = fs::read_to_string(level_path)
			.unwrap_or_else(|err| panic!("Failed to read {}: {}", level_path.display(), err));
		let mut level_json: chipty::LevelDto = serde_json::from_str(&contents)
			.unwrap_or_else(|err| panic!("{} is not valid JSON: {}", level_path.display(), err));

		let prev_author = level_json.author;
		level_json.author = author;

		let changed = level_json.author != prev_author;
		if changed {
			let serialized = serde_json::to_string(&level_json).expect("Failed to serialize level JSON");
			fs::write(level_path, serialized)
				.unwrap_or_else(|err| panic!("Failed to write {}: {}", level_path.display(), err));

			let title = level.metadata.title.as_deref().unwrap_or("<untitled>");
			let rel_path = level_path.strip_prefix(&levelset_dir).unwrap_or(level_path);
			println!("Updated level #{:03} ({}) -> {}", idx + 1, title, rel_path.display());
			updated += 1;
		}
	}

	println!("{} level(s) updated", updated);
}
