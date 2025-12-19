use std::collections::hash_map;
use std::fs;

const LEVELSETS: &[&str] = &["cc1", "cclp1", "cclp2", "cclp3", "cclp4", "cclp5", "Walls_of_CC1"];

fn main() {
	for set_name in LEVELSETS {
		let saves = format!("save/{set_name}/replay");

		let mut records = hash_map::HashMap::new();
		for entry in fs::read_dir(&saves).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.extension().and_then(|s| s.to_str()) != Some("json") {
				continue;
			}

			// File names are like <level{N}.attempt{N}.json>
			let file_stem = path.file_stem().unwrap().to_str().unwrap();
			if !file_stem.starts_with("level") {
				continue;
			}
			let level_id: i32 = file_stem["level".len()..file_stem.find('.').unwrap()].parse().unwrap();

			let replay = fs::read_to_string(&path).unwrap();
			let replay: chipty::ReplayDto = serde_json::from_str(&replay).unwrap();

			// Keep best record only
			match records.entry(level_id) {
				hash_map::Entry::Vacant(v) => {
					v.insert(replay);
				}
				hash_map::Entry::Occupied(mut o) => {
					if replay.ticks < o.get().ticks {
						o.insert(replay);
					}
				}
			}
		}

		// Keep the best records as test cases
		for (level_id, record) in records {
			let test_path = format!("chipcore/tests/replays/{set_name}/level{level_id}.json");
			let write_record = if let Ok(existing_content) = fs::read_to_string(&test_path) {
				let existing: chipty::ReplayDto = serde_json::from_str(&existing_content).unwrap();

				// LevelComplete was changed to be one tick earlier to make it a nice round multiple of BASE_SPD
				// Adjust for this to avoid massive number of replay test failures
				// This is a terrible hack and I hope it won't bite me in the future :)
				let existing_ticks = if existing.ticks % 12 == 1 { existing.ticks - 1 } else { existing.ticks };

				record.ticks < existing_ticks
			}
			else {
				true
			};
			if write_record {
				let serialized = serde_json::to_string_pretty(&record).unwrap();
				fs::write(&test_path, serialized).unwrap();
				println!("{}: Update level{}: {} ticks", set_name, level_id, record.ticks);
			}
		}
	}
}
