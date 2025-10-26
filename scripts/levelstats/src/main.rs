use chipty::*;

struct LevelStats {
	number: usize,
	n_blocks: usize,
}

const LEVEL_PACK: &str = "levelsets/Walls_of_CC1";

fn main() {
	let mut stats = vec![];

	for number in 1..150 {
		let path = format!("{}/lv/level{}.json", LEVEL_PACK, number);
		// dbg!(&path);
		let Ok(content) = std::fs::read_to_string(&path)
		else {
			continue;
		};

		let level: LevelDto = serde_json::from_str(&content).unwrap();
		inspect(number, &level, &mut stats);
	}

	stats.sort_by_key(|s| s.n_blocks);

	for stat in stats.iter() {
		if stat.n_blocks > 1 {
			println!("Level {} has {} blocks", stat.number, stat.n_blocks);
		}
	}
}

fn inspect(number: usize, level: &LevelDto, stats: &mut Vec<LevelStats>) {
	let n_blocks = level.entities.iter().filter(|ent| ent.kind == EntityKind::Block).count();
	stats.push(LevelStats { number, n_blocks });
}
