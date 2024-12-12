use std::collections::HashMap;
use std::{env, fs};

fn test_replay(level: &str, replay: &chipcore::ReplayDto) {
	let seed: u64 = u64::from_str_radix(&replay.seed, 16).unwrap();

	let mut game = chipcore::GameState::default();
	game.parse(level, chipcore::RngSeed::Manual(seed));

	let inputs = chipcore::decode_bytes(&replay.replay);
	for &byte in &inputs {
		let input = chipcore::Input::decode(byte);
		game.tick(&input);
	}

	assert_eq!(game.ps.activity, chipcore::PlayerActivity::Win);
	assert_eq!(game.time, replay.ticks);
	assert_eq!(game.ps.bonks, replay.bonks);
}

#[test]
fn cc1() {
	let replays = include_str!("CC1.json");
	let replays: HashMap<String, chipcore::ReplayDto> = serde_json::from_str(replays).unwrap();

	let dir = env::current_dir().unwrap().parent().unwrap().join("levelsets/cc1/lv");
	for entry in fs::read_dir(dbg!(dir)).unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		let content = fs::read_to_string(&path).unwrap();
		let level: chipcore::FieldDto = serde_json::from_str(&content).unwrap();
		if let Some(replay) = replays.get(&level.name) {
			let bad_rng = level.name == "Blobdance";
			if bad_rng {
				eprintln!("Skipping: {}", level.name);
				continue;
			}
			eprintln!("Now playing: {}", level.name);
			test_replay(&content, replay);
		}
	}
}
