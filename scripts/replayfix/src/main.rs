use std::{fs, io, path, thread, time};
use std::sync::{Arc, atomic, mpsc};
use std::io::Write;

use rayon::prelude::*;

fn brute_force(level: &chipty::LevelDto, replay: &chipty::ReplayDto, seed: u64, count: usize) -> Option<(usize, u64)> {
	let inputs = chipty::decode(&replay.inputs);

	let attempts_done = Arc::new(atomic::AtomicUsize::new(0));
	let best_time = Arc::new(atomic::AtomicI32::new(0));
	let replay_ticks = replay.ticks.max(0);
	let (stop_tx, stop_rx) = mpsc::channel();
	let progress_thread = {
		let attempts_done = Arc::clone(&attempts_done);
		let best_progress = Arc::clone(&best_time);
		thread::spawn(move || {
			let start = time::Instant::now();
			let mut last_line_len = 0usize;
			let mut print_line = |done: bool| {
				let attempts = attempts_done.load(atomic::Ordering::Relaxed);
				let best_ticks = best_progress.load(atomic::Ordering::Relaxed);
				let elapsed = start.elapsed().as_secs_f64().max(1e-6);
				let rate = attempts as f64 / elapsed;
				let message = format!("Trying seeds: best {best_ticks}/{replay_ticks} ticks, {rate:.1} attempts/s");

				if done {
					eprint!("\r{message}\n");
				}
				else {
					let pad_len = last_line_len.saturating_sub(message.len());
					last_line_len = message.len();
					let pad = " ".repeat(pad_len);
					eprint!("\r{message}{pad}");
				}
				_ = io::stderr().flush();
			};

			loop {
				match stop_rx.recv_timeout(time::Duration::from_secs(1)) {
					Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
						print_line(true);
						break;
					}
					Err(mpsc::RecvTimeoutError::Timeout) => {
						print_line(false);
					}
				}
			}
		})
	};

	// Parallel search over the seed range using Rayon.
	let result = (0..count)
		.into_par_iter()
		.find_map_any(|attempt| {
			let mut game = chipcore::GameState::default();
			let local_seed = seed + attempt as u64;

			game.parse(level, chipcore::RngSeed::Manual(local_seed));

			for &byte in &inputs {
				if game.is_game_over() {
					break;
				}
				let input = chipcore::Input::decode(byte);
				game.tick(&input);
			}

			// Find the PlayerGameOver event and assert the reason is LevelComplete
			let events = game.events.take();
			let game_over_reason = events.iter().find_map(|event| match event {
				&chipcore::GameEvent::PlayerGameOver { reason, .. } => Some(reason),
				_ => None,
			});
			let success = game_over_reason == Some(chipcore::GameOverReason::LevelComplete);

			best_time.fetch_max(game.time, atomic::Ordering::Relaxed);
			attempts_done.fetch_add(1, atomic::Ordering::Relaxed);

			if success {
				Some((attempt, local_seed))
			}
			else {
				None
			}
		});

	let _ = stop_tx.send(());
	let _ = progress_thread.join();

	result
}

fn main() {
	let matches = clap::command!()
		.about("Bruteforce a matching RNG seed for a replay")
		.arg(clap::arg!(<LEVEL> "Path to the level JSON").value_parser(clap::value_parser!(path::PathBuf)))
		.arg(clap::arg!(<REPLAY> "Path to the replay JSON").value_parser(clap::value_parser!(path::PathBuf)))
		.arg(clap::arg!(--count [COUNT] "Number of seeds to try (default: 1000)").value_parser(clap::value_parser!(usize)))
		.get_matches();

	let level_path = matches.get_one::<path::PathBuf>("LEVEL").unwrap();
	let replay_path = matches.get_one::<path::PathBuf>("REPLAY").unwrap();
	let count = matches.get_one::<usize>("count").cloned().unwrap_or(1000);

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
