
fn main() {
	for i in 1..150 {
		let path = format!("data/packs/cclp2/lv/level{}.json", i);
		// dbg!(&path);
		let Ok(content) = std::fs::read_to_string(&path)
		else {
			continue;
		};

		let mut data: chipcore::FieldDto = serde_json::from_str(&content).unwrap();
		if fix_level(i, &mut data) {
			let new_content = serde_json::to_string(&data).unwrap();
			std::fs::write(&path, new_content).unwrap();
		}
	}
}

fn fix_level(n: usize, data: &mut chipcore::FieldDto) -> bool {
	let mut fixed = false;

	let mut ents_to_remove = Vec::new();

	// Looking for Block entities targetted by a red connection
	for (ent_index, ent_args) in data.entities.iter().enumerate() {
		if ent_args.kind != chipcore::EntityKind::Block {
			continue;
		}

		let Some(&conn) = data.connections.iter().find(|&conn| conn.dest == ent_args.pos)
		else {
			continue
		};

		{
			let index = (conn.src.y * data.map.width + conn.src.x) as usize;
			let tile = data.map.data[index] as usize;
			let terrain = data.map.legend[tile];
			if terrain != chipcore::Terrain::RedButton {
				continue;
			}
		}

		let new_terrain = match ent_args.face_dir {
			Some(chipcore::Compass::Up) => chipcore::Terrain::CloneBlockN,
			Some(chipcore::Compass::Down) => chipcore::Terrain::CloneBlockS,
			Some(chipcore::Compass::Left) => chipcore::Terrain::CloneBlockW,
			Some(chipcore::Compass::Right) => chipcore::Terrain::CloneBlockE,
			_ => continue,
		};

		eprintln!("Fixing Block at {:?}", ent_args.pos);

		ents_to_remove.push(ent_index);

		let new_tile = {
			if let Some(new_tile) = data.map.legend.iter().position(|&t| t == new_terrain) {
				new_tile as u8
			} else {
				data.map.legend.push(new_terrain);
				data.map.legend.len() as u8 - 1
			}
		};

		let index = (ent_args.pos.y * data.map.width + ent_args.pos.x) as usize;
		data.map.data[index] = new_tile;
		fixed = true;
	}

	for &ent_index in ents_to_remove.iter().rev() {
		data.entities.remove(ent_index);
	}

	if fixed {
		eprintln!("Fixed level{}", n);
	}

	return fixed;
}
