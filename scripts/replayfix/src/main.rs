use std::{fs, path};

use rayon::prelude::*;

fn brute_force(level: &chipty::LevelDto, replay: &chipty::ReplayDto, seed: u64, count: usize) -> Option<(usize, u64)> {
	let inputs = chipty::decode(&replay.replay);

	// Parallel search over the seed range using Rayon.
	(0..count)
		.into_par_iter()
		.find_map_any(|attempt| {
			let mut game = chipcore::GameState::default();
			let local_seed = seed + attempt as u64;

			game.parse(level, chipcore::RngSeed::Manual(local_seed));

			for &byte in &inputs {
				let input = chipcore::Input::decode(byte);
				game.tick(&input);
			}

			let success =
				game.is_game_over() &&
				game.time == replay.ticks &&
				game.ps.bonks == replay.bonks;

			if success {
				Some((attempt, local_seed))
			}
			else {
				None
			}
		})
}

fn main() {
	let app = clap::Command::new("replayfix")
		.about("Bruteforce a matching RNG seed for a replay")
		.arg(clap::arg!(<LEVEL> "Path to the level JSON").allow_invalid_utf8(true))
		.arg(clap::arg!(<REPLAY> "Path to the replay JSON").allow_invalid_utf8(true))
		.arg(clap::arg!(--count [COUNT] "Number of seeds to try (default: 1000)").required(false).takes_value(true))
		.get_matches();

	let level_path = path::PathBuf::from(app.value_of_os("LEVEL").unwrap());
	let replay_path = path::PathBuf::from(app.value_of_os("REPLAY").unwrap());
	let count: usize = app
		.value_of("count")
		.map(|s| s.parse().expect("Invalid count"))
		.unwrap_or(1000);

	let level_content = fs::read_to_string(&level_path).expect("Failed to read level file");
	let replay_content = fs::read_to_string(&replay_path).expect("Failed to read replay file");

	let level: chipty::LevelDto = serde_json::from_str(&level_content).expect("Failed to parse level JSON");
	let replay: chipty::ReplayDto = serde_json::from_str(&replay_content).expect("Failed to parse replay JSON");
	let seed = u64::from_str_radix(&replay.seed, 16).expect("Invalid seed in replay");

	if let Some((attempt, found_seed)) = brute_force(&level, &replay, seed, count) {
		if attempt == 0 {
			println!("Replay already matches the given seed: {found_seed:016x}");
		}
		else {
			println!("Found working seed: {found_seed:016x}");
		}
	}
	else {
		println!("No working seed found after {count} attempts.");
	}
}
