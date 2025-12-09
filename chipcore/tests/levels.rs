use std::fs;

fn test_replay(level: &chipty::LevelDto, replay: &chipty::ReplayDto) {
	let seed: u64 = u64::from_str_radix(&replay.seed, 16).unwrap();

	let mut game = chipcore::GameState::default();
	game.parse(level, chipcore::RngSeed::Manual(seed));

	let inputs = chipty::decode(&replay.inputs);
	for &byte in &inputs {
		if game.is_game_over() {
			panic!("Game ended early during replay");
		}
		let input = chipcore::Input::decode(byte);
		game.tick(&input);
	}

	assert!(game.is_game_over(), "Expected game over at end of replay");
	assert_eq!(game.time, replay.ticks, "Tick count mismatch");
	assert_eq!(game.ps.bonks, replay.bonks, "Bonk count mismatch");
}

#[test]
fn test_levels() {
	for entry in fs::read_dir("tests/levels").unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.extension().and_then(|s| s.to_str()) == Some("json") {
			let level_data = fs::read_to_string(&path).unwrap();
			let level: chipty::LevelDto = serde_json::from_str(&level_data).unwrap();
			if let Some(replays) = &level.replays {
				eprintln!("Playing: \x1b[32m{}\x1b[m", level.name);
				for replay in replays {
					test_replay(&level, replay);
				}
			}
		}
	}
}
