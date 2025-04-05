use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;

use chipcore::{Compass, Conn, EntityArgs, EntityKind, FieldDto, MapDto, Terrain};

fn main() {
	let app = clap::command!("ccdat")
		.arg(clap::arg!(-f <file> "Path to .dat file"))
		.arg(clap::arg!(-n <level> "Level number to extract"));
	let matches = app.get_matches();

	let file_path = matches.value_of("file").expect("No file path provided");
	let level = matches.value_of_t::<i32>("level").expect("Invalid level number");

	let file_data = fs::read(file_path).expect("Failed to read file");
	let view = dataview::DataView::from(&file_data[..]);

	let magic = view.read::<u32>(0);
	let level_count = view.read::<u16>(4) as i32;
	eprintln!("Magic: 0x{:08x}", magic);
	eprintln!("Levels: {}", level_count);
	eprintln!();

	let mut offset = 6;
	for i in 0..level_count {
		let len = view.read::<u16>(offset) as usize;
		offset += 2;

		if i + 1 == level {
			let level_data = view.slice::<u8>(offset, len);
			read_level(level_data);
			break;
		}

		offset += len;
	}
}

fn read_level(data: &[u8]) {
	let data = dataview::DataView::from(data);

	let level_nr = data.read::<u16>(0) as i32;
	let time_limit = data.read::<u16>(2) as i32;
	let chips = data.read::<u16>(4) as i32;
	let upper_layer_len = data.read::<u16>(8) as usize;
	let upper_layer = data.slice::<u8>(10, upper_layer_len);
	let lower_layer_len = data.read::<u16>(10 + upper_layer_len) as usize;
	let lower_layer = data.slice::<u8>(12 + upper_layer_len, lower_layer_len);
	let metadata_len = data.read::<u16>(12 + upper_layer_len + lower_layer_len) as usize;
	let metadata = data.slice::<u8>(14 + upper_layer_len + lower_layer_len, metadata_len);

	eprintln!("Level: {}", level_nr);
	eprintln!("Time limit: {}", time_limit);
	eprintln!("Chips: {}", chips);
	eprintln!("Upper layer: {} bytes", upper_layer_len);
	eprintln!("Lower layer: {} bytes", lower_layer_len);
	eprintln!("Metadata: {} bytes", metadata_len);

	let md = read_metadata(metadata);

	let upper_content = decode_content(upper_layer);
	let lower_content = decode_content(lower_layer);
	let (map, ents, mut conns) = parse_content(&upper_content, &lower_content);

	// let mut conns = Vec::new();
	for lnk in &md.trap_linkage {
		conns.push(*lnk);
	}
	for lnk in &md.cloner_linkage {
		conns.push(*lnk);
	}

	let field = FieldDto {
		name: md.title,
		author: md.author,
		hint: md.hint,
		password: Some(md.password),
		time_limit,
		required_chips: chips,
		map,
		entities: ents,
		connections: conns,
		replays: None,
	};

	let json = serde_json::to_string(&field).unwrap();
	print!("{}", json);
}

fn decode_content(data: &[u8]) -> Vec<u8> {
	let mut tiles = Vec::new();
	let mut offset = 0;
	while offset < data.len() {
		if data[offset] == 0xff {
			let count = data[offset + 1] as usize;
			let tile = data[offset + 2];
			tiles.extend(std::iter::repeat(tile).take(count));
			offset += 3;
		}
		else {
			tiles.push(data[offset]);
			offset += 1;
		}
	}
	return tiles;
}

fn process_tile(terrain: &mut Vec<Terrain>, entities: &mut Vec<EntityArgs>, pos: cvmath::Vec2<i32>, tile: u8) {
	let index = (pos.y * 32 + pos.x) as usize;
	match tile {
		0x00 => (),//terrain[index] = Terrain::Floor,
		0x01 => terrain[index] = Terrain::Wall,
		0x02 => entities.push(ent_args(EntityKind::Chip, pos, None)),
		0x03 => terrain[index] = Terrain::Water,
		0x04 => terrain[index] = Terrain::Fire,
		0x05 => terrain[index] = Terrain::InvisibleWall,
		0x06 => terrain[index] = Terrain::ThinWallN,
		0x07 => terrain[index] = Terrain::ThinWallW,
		0x08 => terrain[index] = Terrain::ThinWallS,
		0x09 => terrain[index] = Terrain::ThinWallE,
		0x0a => entities.push(ent_args(EntityKind::Block, pos, None)),
		0x0b => terrain[index] = Terrain::Dirt,
		0x0c => terrain[index] = Terrain::Ice,
		0x0d => terrain[index] = Terrain::ForceS,
		0x0e => terrain[index] = Terrain::CloneBlockN,
		0x0f => terrain[index] = Terrain::CloneBlockW,

		0x10 => terrain[index] = Terrain::CloneBlockS,
		0x11 => terrain[index] = Terrain::CloneBlockE,
		0x12 => terrain[index] = Terrain::ForceN,
		0x13 => terrain[index] = Terrain::ForceE,
		0x14 => terrain[index] = Terrain::ForceW,
		0x15 => terrain[index] = Terrain::Exit,
		0x16 => terrain[index] = Terrain::BlueLock,
		0x17 => terrain[index] = Terrain::RedLock,
		0x18 => terrain[index] = Terrain::GreenLock,
		0x19 => terrain[index] = Terrain::YellowLock,
		0x1a => terrain[index] = Terrain::IceNW,
		0x1b => terrain[index] = Terrain::IceNE,
		0x1c => terrain[index] = Terrain::IceSE,
		0x1d => terrain[index] = Terrain::IceSW,
		0x1e => terrain[index] = Terrain::FakeBlueWall,
		0x1f => terrain[index] = Terrain::RealBlueWall,

		0x21 => entities.push(ent_args(EntityKind::Thief, pos, None)),
		0x22 => entities.push(ent_args(EntityKind::Socket, pos, None)),
		0x23 => terrain[index] = Terrain::GreenButton,
		0x24 => terrain[index] = Terrain::RedButton,
		0x25 => terrain[index] = Terrain::ToggleWall,
		0x26 => terrain[index] = Terrain::ToggleFloor,
		0x27 => terrain[index] = Terrain::BrownButton,
		0x28 => terrain[index] = Terrain::BlueButton,
		0x29 => terrain[index] = Terrain::Teleport,
		0x2a => entities.push(ent_args(EntityKind::Bomb, pos, None)),
		0x2b => terrain[index] = Terrain::BearTrap,
		0x2c => terrain[index] = Terrain::HiddenWall,
		0x2d => terrain[index] = Terrain::Gravel,
		0x2e => terrain[index] = Terrain::RecessedWall,
		0x2f => terrain[index] = Terrain::Hint,

		0x30 => terrain[index] = Terrain::ThinWallSE,
		0x31 => terrain[index] = Terrain::CloneMachine,
		0x32 => terrain[index] = Terrain::ForceRandom,
		0x33 => terrain[index] = Terrain::WaterHazard, // Drowned chip
		0x39 => terrain[index] = Terrain::FakeExit,
		0x3a => terrain[index] = Terrain::FakeExit,
		0x3b => terrain[index] = Terrain::FakeExit,

		0x40 => entities.push(ent_args(EntityKind::Bug, pos, Some(Compass::Up))),
		0x41 => entities.push(ent_args(EntityKind::Bug, pos, Some(Compass::Left))),
		0x42 => entities.push(ent_args(EntityKind::Bug, pos, Some(Compass::Down))),
		0x43 => entities.push(ent_args(EntityKind::Bug, pos, Some(Compass::Right))),

		0x44 => entities.push(ent_args(EntityKind::FireBall, pos, Some(Compass::Up))),
		0x45 => entities.push(ent_args(EntityKind::FireBall, pos, Some(Compass::Left))),
		0x46 => entities.push(ent_args(EntityKind::FireBall, pos, Some(Compass::Down))),
		0x47 => entities.push(ent_args(EntityKind::FireBall, pos, Some(Compass::Right))),

		0x48 => entities.push(ent_args(EntityKind::PinkBall, pos, Some(Compass::Up))),
		0x49 => entities.push(ent_args(EntityKind::PinkBall, pos, Some(Compass::Left))),
		0x4a => entities.push(ent_args(EntityKind::PinkBall, pos, Some(Compass::Down))),
		0x4b => entities.push(ent_args(EntityKind::PinkBall, pos, Some(Compass::Right))),

		0x4c => entities.push(ent_args(EntityKind::Tank, pos, Some(Compass::Up))),
		0x4d => entities.push(ent_args(EntityKind::Tank, pos, Some(Compass::Left))),
		0x4e => entities.push(ent_args(EntityKind::Tank, pos, Some(Compass::Down))),
		0x4f => entities.push(ent_args(EntityKind::Tank, pos, Some(Compass::Right))),

		0x50 => entities.push(ent_args(EntityKind::Glider, pos, Some(Compass::Up))),
		0x51 => entities.push(ent_args(EntityKind::Glider, pos, Some(Compass::Left))),
		0x52 => entities.push(ent_args(EntityKind::Glider, pos, Some(Compass::Down))),
		0x53 => entities.push(ent_args(EntityKind::Glider, pos, Some(Compass::Right))),

		0x54 => entities.push(ent_args(EntityKind::Teeth, pos, Some(Compass::Up))),
		0x55 => entities.push(ent_args(EntityKind::Teeth, pos, Some(Compass::Left))),
		0x56 => entities.push(ent_args(EntityKind::Teeth, pos, Some(Compass::Down))),
		0x57 => entities.push(ent_args(EntityKind::Teeth, pos, Some(Compass::Right))),

		0x58 => entities.push(ent_args(EntityKind::Walker, pos, Some(Compass::Up))),
		0x59 => entities.push(ent_args(EntityKind::Walker, pos, Some(Compass::Left))),
		0x5a => entities.push(ent_args(EntityKind::Walker, pos, Some(Compass::Down))),
		0x5b => entities.push(ent_args(EntityKind::Walker, pos, Some(Compass::Right))),

		0x5c => entities.push(ent_args(EntityKind::Blob, pos, Some(Compass::Up))),
		0x5d => entities.push(ent_args(EntityKind::Blob, pos, Some(Compass::Left))),
		0x5e => entities.push(ent_args(EntityKind::Blob, pos, Some(Compass::Down))),
		0x5f => entities.push(ent_args(EntityKind::Blob, pos, Some(Compass::Right))),

		0x60 => entities.push(ent_args(EntityKind::Paramecium, pos, Some(Compass::Up))),
		0x61 => entities.push(ent_args(EntityKind::Paramecium, pos, Some(Compass::Left))),
		0x62 => entities.push(ent_args(EntityKind::Paramecium, pos, Some(Compass::Down))),
		0x63 => entities.push(ent_args(EntityKind::Paramecium, pos, Some(Compass::Right))),

		0x64 => entities.push(ent_args(EntityKind::BlueKey, pos, None)),
		0x65 => entities.push(ent_args(EntityKind::RedKey, pos, None)),
		0x66 => entities.push(ent_args(EntityKind::GreenKey, pos, None)),
		0x67 => entities.push(ent_args(EntityKind::YellowKey, pos, None)),
		0x68 => entities.push(ent_args(EntityKind::Flippers, pos, None)),
		0x69 => entities.push(ent_args(EntityKind::FireBoots, pos, None)),
		0x6a => entities.push(ent_args(EntityKind::IceSkates, pos, None)),
		0x6b => entities.push(ent_args(EntityKind::SuctionBoots, pos, None)),
		0x6c => entities.push(ent_args(EntityKind::Player, pos, None)),
		0x6d => entities.push(ent_args(EntityKind::Player, pos, None)),
		0x6e => entities.push(ent_args(EntityKind::Player, pos, None)),
		0x6f => entities.push(ent_args(EntityKind::Player, pos, None)),
		value => unimplemented!("Tile: ${:02x}", value),
	}
}

fn parse_content(upper: &[u8], lower: &[u8]) -> (MapDto, Vec<EntityArgs>, Vec<Conn>) {
	let mut terrain = vec![Terrain::Floor; 32 * 32];
	let mut entities = Vec::new();

	for y in 0..32 {
		for x in 0..32 {
			let index = y * 32 + x;
			let pos = cvmath::Vec2i(x as i32, y as i32);

			process_tile(&mut terrain, &mut entities, pos, lower[index]);
			process_tile(&mut terrain, &mut entities, pos, upper[index]);
		}
	}

	let mut conns = Vec::new();
	let mut last_teleport = None;
	let mut prev_teleport = None;
	for y in (0..32).rev() {
		for x in (0..32).rev() {
			let index = y as usize * 32 + x as usize;
			let pos = cvmath::Vec2i(x as i32, y as i32);

			if terrain[index] == Terrain::Teleport {
				if last_teleport.is_none() {
					last_teleport = Some(pos);
				}
				if let Some(prev_teleport) = prev_teleport {
					conns.push(Conn { src: prev_teleport, dest: pos });
				}
				prev_teleport = Some(pos);
			}
		}
	}
	if let Some(last_teleport) = last_teleport {
		if let Some(prev_teleport) = prev_teleport {
			conns.push(Conn { src: prev_teleport, dest: last_teleport });
		}
	}

	let mut legend = Vec::new();
	let data = {
		let mut legend_map = HashMap::new();
		legend_map.insert(Terrain::Blank, 0); legend.push(Terrain::Blank);
		legend_map.insert(Terrain::Floor, 1); legend.push(Terrain::Floor);
		let mut idx = 2;
		for &terrain in terrain.iter() {
			if terrain == Terrain::Teleport {
				eprintln!("---- TELEPORT DETECTED!! ----");
			}
			if !legend_map.contains_key(&terrain) {
				legend_map.insert(terrain, idx);
				legend.push(terrain);
				idx += 1;
			}
		}
		terrain.iter().map(|&terrain| legend_map[&terrain]).collect()
	};

	let map = MapDto { width: 32, height: 32, data, legend };
	return (map, entities, conns);
}

fn ent_args(kind: EntityKind, pos: cvmath::Vec2i, face_dir: Option<Compass>) -> EntityArgs {
	EntityArgs { kind, pos, face_dir }
}

#[allow(dead_code)]
struct Metadata {
	time_limit: i32,
	required_chips: i32,
	title: String,
	trap_linkage: Vec<Conn>,
	cloner_linkage: Vec<Conn>,
	password: String,
	hint: Option<String>,
	author: Option<String>,
}

fn read_metadata(data: &[u8]) -> Metadata {
	let mut time_limit = 0;
	let mut required_chips = 0;
	let mut title = String::new();
	let mut trap_linkage = Vec::new();
	let mut cloner_linkage = Vec::new();
	let mut password = String::new();
	let mut hint = None;
	let mut author = None;

	let mut i = 0;
	while i < data.len() {
		let ty = data[i];
		let len = data[i + 1] as usize;
		i += 2;

		let view = dataview::DataView::from(&data[i..i + len]);

		match ty {
			1 => {
				time_limit = view.read::<u16>(0) as i32;
				eprintln!("Time limit: {}", time_limit);
			}
			2 => {
				required_chips = view.read::<u16>(0) as i32;
				eprintln!("Required chips: {}", required_chips);
			}
			3 => {
				let title_ = CStr::from_bytes_with_nul(view.slice(0, len)).unwrap();
				title = title_.to_str().unwrap().to_string();
				eprintln!("Title: {}", title);
			}
			4 => {
				let mut j = 0;
				while j < len {
					let brown_x = view.read::<u16>(j);
					let brown_y = view.read::<u16>(j + 2);
					let trap_x = view.read::<u16>(j + 4);
					let trap_y = view.read::<u16>(j + 6);
					let src = cvmath::Vec2i(brown_x as i32, brown_y as i32);
					let dest = cvmath::Vec2i(trap_x as i32, trap_y as i32);
					trap_linkage.push(Conn { src, dest });
					j += 10;
				}
			}
			5 => {
				let mut j = 0;
				while j < len {
					let red_x = view.read::<u16>(j);
					let red_y = view.read::<u16>(j + 2);
					let cloner_x = view.read::<u16>(j + 4);
					let cloner_y = view.read::<u16>(j + 6);
					let src = cvmath::Vec2i(red_x as i32, red_y as i32);
					let dest = cvmath::Vec2i(cloner_x as i32, cloner_y as i32);
					cloner_linkage.push(Conn { src, dest });
					j += 8;
				}
			}
			6 => {
				let mut bytes = view.slice::<u8>(0, len).to_vec();
				if bytes.len() > 0 {
					for k in 0..bytes.len() - 1 {
						bytes[k] ^= 0x99;
					}
				}
				let password_ = CStr::from_bytes_with_nul(&bytes).unwrap();
				password = password_.to_str().unwrap().to_string();
				eprintln!("Password: {}", password);
			}
			7 => {
				let hint_ = CStr::from_bytes_with_nul(view.slice(0, len)).unwrap();
				hint = Some(hint_.to_str().unwrap().to_string());
				eprintln!("Hint: {:?}", hint);
			}
			9 => {
				let author_ = CStr::from_bytes_with_nul(view.slice(0, len)).unwrap();
				author = Some(author_.to_string_lossy().to_string());
				eprintln!("Author: {:?}", author);
			}
			10 => eprintln!("Monster list"),
			ty => unimplemented!("Metadata type: {}", ty),
		}

		i += len;
	}

	Metadata {
		time_limit,
		required_chips,
		title,
		trap_linkage,
		cloner_linkage,
		password,
		hint,
		author,
	}
}
