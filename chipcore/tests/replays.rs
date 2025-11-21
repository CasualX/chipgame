use std::{env, fs};
use std::path::Path;

fn test_replay(level: &chipty::LevelDto, replay: &chipty::ReplayDto) -> bool {
	let seed: u64 = u64::from_str_radix(&replay.seed, 16).unwrap();

	let mut game = chipcore::GameState::default();
	game.parse(level, chipcore::RngSeed::Manual(seed));

	let inputs = chipty::decode(&replay.replay);
	for &byte in &inputs {
		let input = chipcore::Input::decode(byte);
		game.tick(&input);
	}

	let mut success = true;
	if !game.is_game_over() {
		eprintln!(" - game over mismatch: expected game over");
		success = false;
	}
	if game.time != replay.ticks {
		eprintln!(" - ticks mismatch: expected {}, got {}", replay.ticks, game.time);
		success = false;
	}
	if game.ps.bonks != replay.bonks {
		eprintln!(" - bonks mismatch: expected {}, got {}", replay.bonks, game.ps.bonks);
		success = false;
	}

	// // Try brute-forcing the seed if the replay failed due to RNG changes
	// if !success {
	// 	brute_force(level, replay, activity, seed, 1000);
	// }

	success
}

#[allow(dead_code)]
fn brute_force(level: &chipty::LevelDto, replay: &chipty::ReplayDto, mut seed: u64, count: usize) -> bool {
	let mut success = false;
	for _ in 0..count {
		seed += 1;
		let mut game = chipcore::GameState::default();
		game.parse(level, chipcore::RngSeed::Manual(seed));

		let inputs = chipty::decode(&replay.replay);
		for &byte in &inputs {
			let input = chipcore::Input::decode(byte);
			game.tick(&input);
		}

		if !game.is_game_over() {
			continue;
		}
		if game.time != replay.ticks {
			continue;
		}
		if game.ps.bonks != replay.bonks {
			continue;
		}

		success = true;
		eprintln!(" - succeeded with seed {:016x}", seed);
		break;
	}

	success
}

fn test_levelset(levels_dir: &Path, replays_dir: &Path) {
	eprintln!("\x1b[33mLevels\x1b[m: {:?}", levels_dir);
	eprintln!("\x1b[33mReplays\x1b[m: {:?}", replays_dir);
	let mut fail_count = 0usize;
	for level_number in 1..150 {
		let level_path = levels_dir.join(format!("level{level_number}.json"));
		let level_content = fs::read_to_string(&level_path).unwrap();
		let level: chipty::LevelDto = serde_json::from_str(&level_content).unwrap();
		let password = level.password.as_deref().unwrap_or("<unknown>");
		let replay_path = replays_dir.join(level_path.file_name().unwrap());
		if let Ok(replay_content) = fs::read_to_string(&replay_path) {
			let replay: chipty::ReplayDto = serde_json::from_str(&replay_content).unwrap();
			eprintln!("Playing: level{level_number} {password:?}: \x1b[32m{}\x1b[m", level.name);
			fail_count += !test_replay(&level, &replay) as usize;
		}
		else {
			eprintln!("\x1b[31mSkipped\x1b[m: level{level_number} {password:?}: \x1b[32m{}\x1b[m", level.name);
		}
	}
	if fail_count > 0 {
		panic!("{} replay(s) failed", fail_count);
	}
}

fn test_level(levels_dir: &Path, replays_dir: &Path, level_name: &str) {
	let level_path = levels_dir.join(format!("{level_name}.json"));
	let level_content = fs::read_to_string(&level_path).unwrap();
	let level: chipty::LevelDto = serde_json::from_str(&level_content).unwrap();
	let replay_path = replays_dir.join(level_path.file_name().unwrap());
	if let Ok(replay_content) = fs::read_to_string(&replay_path) {
		let replay: chipty::ReplayDto = serde_json::from_str(&replay_content).unwrap();
		eprintln!("Playing {:?}: \x1b[32m{}\x1b[m", level_name, level.name);
		if !test_replay(&level, &replay) {
			panic!("replay failed for {:?}", level_name);
		}
	}
	else {
		eprintln!("\x1b[31mSkipped\x1b[m {:?}: \x1b[32m{}\x1b[m", level_name, level.name);
	}
}

#[test]
fn cc1() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/cc1/lv");
	let replays_dir = current_dir.join("tests/replays/cc1");
	test_levelset(&levels_dir, &replays_dir);
}

#[test]
fn cclp1() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/cclp1/lv");
	let replays_dir = current_dir.join("tests/replays/cclp1");
	test_levelset(&levels_dir, &replays_dir);
}

#[test]
fn cclp3() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/cclp3/lv");
	let replays_dir = current_dir.join("tests/replays/cclp3");
	test_levelset(&levels_dir, &replays_dir);
}

#[test]
fn cclp4() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/cclp4/lv");
	let replays_dir = current_dir.join("tests/replays/cclp4");
	test_levelset(&levels_dir, &replays_dir);
}

#[test]
fn cclp5() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/cclp5/lv");
	let replays_dir = current_dir.join("tests/replays/cclp5");
	test_levelset(&levels_dir, &replays_dir);
}

#[test]
fn dev() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/dev");
	let replays_dir = current_dir.join("tests/replays/dev");
	test_level(&levels_dir, &replays_dir, "iceblock");
}

#[test]
fn walls_of_cc1() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/Walls_of_CC1/lv");
	let replays_dir = current_dir.join("tests/replays/Walls_of_CC1");
	test_levelset(&levels_dir, &replays_dir);
}
