use std::{env, fs};
use std::path::Path;

fn play(level: &chipty::LevelDto, replay: &chipty::ReplayDto) -> bool {
	let seed: u64 = u64::from_str_radix(&replay.seed, 16).unwrap();
	let inputs = chipty::decode(&replay.inputs);

	let mut game = chipcore::GameState::default();
	game.parse(level, chipcore::RngSeed::Manual(seed));

	let mut i = 0;
	while i < inputs.len() {
		if game.is_game_over() {
			break;
		}
		game.tick(&chipcore::Input::decode(inputs[i]));
		i += 1;
	}

	// Find the PlayerGameOver event and assert the reason is LevelComplete
	let events = game.events.take();
	let game_over_reason = events.iter().find_map(|event| match event {
		&chipcore::GameEvent::PlayerGameOver { reason, .. } => Some(reason),
		_ => None,
	});
	if game_over_reason == Some(chipcore::GameOverReason::LevelComplete) {
		return true;
	}

	eprintln!(" - FAILED at {} after {i}/{} inputs: expected LevelComplete, got {game_over_reason:?}", chipcore::FmtTime(game.time), inputs.len());
	return false;
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
			fail_count += !play(&level, &replay) as usize;
		}
		else {
			eprintln!("\x1b[31mSkipped\x1b[m: level{level_number} {password:?}: \x1b[32m{}\x1b[m", level.name);
		}
	}
	if fail_count > 0 {
		panic!("{} replay(s) failed", fail_count);
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
fn cclp2() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/cclp2/lv");
	let replays_dir = current_dir.join("tests/replays/cclp2");
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
fn walls_of_cc1() {
	let current_dir = env::current_dir().unwrap();
	let levels_dir = current_dir.parent().unwrap().join("levelsets/Walls_of_CC1/lv");
	let replays_dir = current_dir.join("tests/replays/Walls_of_CC1");
	test_levelset(&levels_dir, &replays_dir);
}
