
struct LevelStats {
	level: usize,
	n_blocks: usize,
}

const LEVEL_PACK: &str = "data/packs/cclp1";

fn main() {
	let mut stats = vec![];

	for i in 1..150 {
		let path = format!("{}/lv/level{}.json", LEVEL_PACK, i);
		// dbg!(&path);
		let Ok(content) = std::fs::read_to_string(&path)
		else {
			continue;
		};

		let data: chipcore::FieldDto = serde_json::from_str(&content).unwrap();
		inspect(i, &data, &mut stats);
	}

	stats.sort_by_key(|s| s.n_blocks);

	for stat in stats.iter() {
		if stat.n_blocks > 1 {
			println!("Level {} has {} blocks", stat.level, stat.n_blocks);
		}
	}
}

fn inspect(level: usize, data: &chipcore::FieldDto, stats: &mut Vec<LevelStats>) {
	let n_blocks = data.entities.iter().filter(|ent| ent.kind == chipcore::EntityKind::Block).count();
	stats.push(LevelStats { level, n_blocks });
}
