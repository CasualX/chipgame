
#[test]
fn replays() {
	let cc1_save = include_str!("CC1.json");
	let cc1: chipgame::savedto::SaveDto = serde_json::from_str(cc1_save).unwrap();

	eprintln!("current_dir: {:?}", std::env::current_dir().unwrap());
	let _ = std::env::set_current_dir(std::env::current_dir().unwrap().parent().unwrap());
	let mut lvsets = chipgame::play::LevelSets::default();
	lvsets.load();
	lvsets.selected = lvsets.collection.iter().position(|x| x.name == "CC1").unwrap();

	for (name, entry) in &cc1.records.mintime {

		let seed: u64 = u64::from_str_radix(&entry.seed, 16).unwrap();

		let level_number = lvsets.current().get_level_number(name).unwrap();

		let level_data = &lvsets.current().lv_data[level_number as usize - 1];

		let mut game = chipcore::GameState::default();
		game.parse(&level_data, chipcore::RngSeed::Manual(seed));

		let inputs = chipgame::play::decode_bytes(&entry.replay);

		for &byte in &inputs {
			let input = chipcore::Input::decode(byte);
			game.tick(&input);
		}

		assert_eq!(game.time, entry.ticks);
		assert_eq!(game.ps.activity, chipcore::PlayerActivity::Win);
	}
}

