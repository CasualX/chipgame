use std::{collections::HashMap, fs, path};

fn decode_html_entities(value: &str) -> String {
	let mut out = value.replace("&quot;", "\"");
	out = out.replace("&apos;", "'");
	out = out.replace("&lt;", "<");
	out = out.replace("&gt;", ">");
	out = out.replace("&amp;", "&");
	out
}

fn main() {
	let matches = clap::command!()
		.about("Backfill author metadata in cclp3 levels using CCLP3.ccx")
		.arg(
			clap::arg!(<CCX_FILE> "Path to CCLP3.ccx")
				.value_parser(clap::value_parser!(path::PathBuf)),
		)
		.arg(
			clap::arg!(<LEVELSET_DIR> "Path to the unpacked cclp3 levelset directory")
				.value_parser(clap::value_parser!(path::PathBuf)),
		)
		.get_matches();

	let ccx_path = matches.get_one::<path::PathBuf>("CCX_FILE").unwrap().clone();
	let levelset_dir = matches
		.get_one::<path::PathBuf>("LEVELSET_DIR")
		.unwrap()
		.clone();

	let ccx_contents = fs::read_to_string(&ccx_path)
		.unwrap_or_else(|err| panic!("Failed to read {}: {}", ccx_path.display(), err));
	let level_re = regex::Regex::new(
		r#"<level\b[^>]*\bnumber=\"(\d+)\"[^>]*\bauthor=\"([^\"]*)\""#,
	)
	.expect("Failed to compile level regex");

	let mut authors_by_level: HashMap<usize, Option<String>> = HashMap::new();
	for cap in level_re.captures_iter(&ccx_contents) {
		let level_num: usize = cap
			.get(1)
			.expect("Missing level number")
			.as_str()
			.parse()
			.expect("Invalid level number");
		let author_raw = cap.get(2).expect("Missing author").as_str();
		let author = decode_html_entities(author_raw)
			.trim()
			.to_string();
		let author = if author.is_empty() { None } else { Some(author) };
		authors_by_level.insert(level_num, author);
	}

	let index_path = levelset_dir.join("index.json");
	let index_data = fs::read_to_string(&index_path)
		.unwrap_or_else(|err| panic!("Failed to read {}: {}", index_path.display(), err));
	let levelset: chipty::LevelSetDto =
		serde_json::from_str(&index_data).expect("Invalid levelset index");

	let mut level_files = Vec::new();
	for (i, level_ref) in levelset.levels.iter().enumerate() {
		match level_ref {
			chipty::LevelRef::Indirect(file) => level_files.push(levelset_dir.join(file)),
			chipty::LevelRef::Direct(_) => {
				panic!("Level {} stored inline; run on an indirect levelset", i + 1)
			}
		}
	}

	let mut updated = 0usize;
	for (idx, level_path) in level_files.iter().enumerate() {
		let level_num = idx + 1;
		let author = authors_by_level.get(&level_num).cloned().unwrap_or(None);
		eprintln!("Level{}: author = {:?}", level_num, author);

		let contents = fs::read_to_string(level_path)
			.unwrap_or_else(|err| panic!("Failed to read {}: {}", level_path.display(), err));
		let mut level_json: chipty::LevelDto = serde_json::from_str(&contents)
			.unwrap_or_else(|err| panic!("{} is not valid JSON: {}", level_path.display(), err));

		let prev_author = level_json.author.clone();
		level_json.author = author;

		let changed = level_json.author != prev_author;
		if changed {
			let serialized =
				serde_json::to_string(&level_json).expect("Failed to serialize level JSON");
			fs::write(level_path, serialized)
				.unwrap_or_else(|err| panic!("Failed to write {}: {}", level_path.display(), err));

			let rel_path = level_path.strip_prefix(&levelset_dir).unwrap_or(level_path);
			println!("Updated level #{:03} -> {}", level_num, rel_path.display());
			updated += 1;
		}
	}

	println!("{} level(s) updated", updated);
}
