use std::{cmp, fmt, fs, path};

struct LevelStats {
	level_number: usize,
	level_name: String,
	quantity: usize,
}

fn main() {
	let matches = clap::command!()
		.about("Find levels with certain properties")
		.arg(clap::arg!(<PATH> "Path to the levelpack").value_parser(clap::value_parser!(path::PathBuf)))
		// .arg(clap::arg!(-k --key [KEY] "Encryption key").required(false))
		.arg(clap::arg!(--terrain [TERRAIN] "Terrain type to filter by"))
		.arg(clap::arg!(--entity [ENTITY] "Entity type to filter by"))
		.arg(clap::arg!(--asc "Sort results in ascending order").required(false).action(clap::ArgAction::SetTrue))
		.get_matches();

	let path = matches.get_one::<path::PathBuf>("PATH").unwrap().clone();
	let terrain: Option<chipty::Terrain> = matches.get_one::<String>("terrain").map(|s| s.parse().expect("Invalid terrain type"));
	let entity: Option<chipty::EntityKind> = matches.get_one::<String>("entity").map(|s| s.parse().expect("Invalid entity type"));
	let asc = matches.get_flag("asc");
	if terrain.is_some() && entity.is_some() {
		panic!("Cannot filter by both terrain and entity at the same time");
	}
	if terrain.is_none() && entity.is_none() {
		panic!("Must specify either terrain or entity to filter by");
	}

	let mut stats = vec![];

	for number in 1..150 {
		let path = path.join(format!("lv/level{}.json", number));
		let Ok(content) = fs::read_to_string(&path)
		else {
			continue;
		};

		let level: chipty::LevelDto = serde_json::from_str(&content).unwrap();

		let quantity = if let Some(terrain) = terrain {
			if let Some(tile) = level.map.legend.iter().position(|&t| t == terrain) {
				level.map.data.iter().filter(|&&t| t as usize == tile).count()
			}
			else {
				0
			}
		}
		else if let Some(entity) = entity {
			level.entities.iter().filter(|ent| ent.kind == entity).count()
		}
		else {
			unreachable!()
		};
		stats.push(LevelStats { level_number: number, level_name: level.name.clone(), quantity });
	}

	if asc {
		stats.sort_by_key(|s| s.quantity);
	}
	else {
		stats.sort_by_key(|s| cmp::Reverse(s.quantity));
	}

	let label: &dyn fmt::Debug = if let Some(terrain) = &terrain { terrain }
	else if let Some(entity) = &entity { entity }
	else { unreachable!() };

	let mut last = None;
	for stat in stats.iter() {
		if stat.quantity > 1 {
			println!("Level {} has {} {:?}s — {:?}", stat.level_number, stat.quantity, label, stat.level_name);
			last = Some(stat);
		}
	}

	if let Some(last) = last {
		eprintln!("cargo run --bin chipedit -- {}", path.join(format!("lv/level{}.json", last.level_number)).display());
	}
}
