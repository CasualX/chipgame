use std::fs;
use chipty::*;

const LEVEL_SET: &str = "levelsets/cclp1";

fn main() {
	for number in 1..150 {
		let path = format!("{}/lv/level{}.json", LEVEL_SET, number);
		// dbg!(&path);
		let Ok(content) = fs::read_to_string(&path)
		else {
			continue;
		};

		let mut level: LevelDto = serde_json::from_str(&content).unwrap();
		if fix_level(number, &mut level) {
			let new_content = serde_json::to_string(&level).unwrap();
			fs::write(&path, new_content).unwrap();
		}
	}
}

fn fix_level(number: i32, level: &mut LevelDto) -> bool {
	let mut fixed = false;

	let mut ents_to_remove = Vec::new();

	// Looking for Block entities targetted by a red connection
	for (ent_index, ent_args) in level.entities.iter().enumerate() {
		if ent_args.kind != EntityKind::Block {
			continue;
		}

		let Some(&conn) = level.connections.iter().find(|&conn| conn.dest == ent_args.pos)
		else {
			continue
		};

		{
			let index = (conn.src.y * level.map.width + conn.src.x) as usize;
			let tile = level.map.data[index] as usize;
			let terrain = level.map.legend[tile];
			if terrain != Terrain::RedButton {
				continue;
			}
		}

		let new_terrain = match ent_args.face_dir {
			Some(Compass::Up) => Terrain::CloneBlockN,
			Some(Compass::Down) => Terrain::CloneBlockS,
			Some(Compass::Left) => Terrain::CloneBlockW,
			Some(Compass::Right) => Terrain::CloneBlockE,
			_ => continue,
		};

		eprintln!("Fixing Block at {:?}", ent_args.pos);

		ents_to_remove.push(ent_index);

		let new_tile = {
			if let Some(new_tile) = level.map.legend.iter().position(|&t| t == new_terrain) {
				new_tile as u8
			} else {
				level.map.legend.push(new_terrain);
				level.map.legend.len() as u8 - 1
			}
		};

		let index = (ent_args.pos.y * level.map.width + ent_args.pos.x) as usize;
		level.map.data[index] = new_tile;
		fixed = true;
	}

	for &ent_index in ents_to_remove.iter().rev() {
		level.entities.remove(ent_index);
	}

	if fixed {
		eprintln!("Fixed level{}", number);
	}

	return fixed;
}
