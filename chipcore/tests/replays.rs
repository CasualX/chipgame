use std::{env, fs};
use std::path::Path;

fn test_replay(level: &str, replay: &chipty::ReplayDto, activity: chipcore::PlayerActivity) {
	let seed: u64 = u64::from_str_radix(&replay.seed, 16).unwrap();

	let mut game = chipcore::GameState::default();
	game.parse(level, chipcore::RngSeed::Manual(seed));

	let inputs = chipty::decode_bytes(&replay.replay);
	for &byte in &inputs {
		let input = chipcore::Input::decode(byte);
		game.tick(&input);
	}

	assert_eq!(game.ps.activity, activity);
	assert_eq!(game.time, replay.ticks);
	assert_eq!(game.ps.bonks, replay.bonks);
}

fn test_levelset(levels_dir: &Path, replays_dir: &Path) {
	eprintln!("\x1b[33mLevels\x1b[m: {:?}", levels_dir);
	eprintln!("\x1b[33mReplays\x1b[m: {:?}", replays_dir);
	for level_number in 1..150 {
		let level_path = levels_dir.join(format!("level{level_number}.json"));
		let level_content = fs::read_to_string(&level_path).unwrap();
		let level: chipcore::LevelDto = serde_json::from_str(&level_content).unwrap();
		let replay_path = replays_dir.join(level_path.file_name().unwrap());
		if let Ok(replay_content) = fs::read_to_string(&replay_path) {
			let replay: chipty::ReplayDto = serde_json::from_str(&replay_content).unwrap();
			eprintln!("Playing: level{} {:?}: \x1b[32m{}\x1b[m", level_number, level.password.unwrap(), level.name);
			test_replay(&level_content, &replay, chipcore::PlayerActivity::Win);
		}
		else {
			eprintln!("\x1b[31mSkipped\x1b[m: level{} {:?}: \x1b[32m{}\x1b[m", level_number, level.password.unwrap(), level.name);
		}
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
