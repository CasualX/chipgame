use std::path::PathBuf;

use super::*;

pub use chipty::{LevelRef, LevelSetDto};

#[derive(Default)]
pub struct LevelSet {
	pub name: String,
	pub title: String,
	pub about: Option<String>,
	pub splash: Option<Vec<u8>>,
	pub levels: Vec<chipty::LevelDto>,
}
impl LevelSet {
	pub fn get_level_number(&self, name: &str) -> Option<i32> {
		self.levels.iter().position(|level| level.name == name).map(|i| i as i32 + 1)
	}
	pub fn get_level_index(&self, name: &str) -> Option<usize> {
		self.levels.iter().position(|level| level.name == name)
	}
}

pub struct LevelSets {
	pub selected: i32,
	pub collection: Vec<LevelSet>,
}
impl Default for LevelSets {
	fn default() -> Self {
		Self {
			selected: -1,
			collection: Vec::new(),
		}
	}
}
impl LevelSets {
	pub fn current(&self) -> &LevelSet {
		&self.collection[self.selected as usize]
	}
	pub fn load(&mut self) {
		load_levelsets(&mut self.collection);
	}
}

fn load_levelsets(sets: &mut Vec<LevelSet>) {
	let dir = match fs::read_dir("levelsets") {
		Ok(dir) => dir,
		Err(err) => {
			eprintln!("Error reading levelsets directory: {}", err);
			return;
		}
	};
	for entry in dir {
		match entry {
			Ok(entry) => {
				let path = entry.path();
				if path.is_dir() {
					let fs = FileSystem::StdFs(path.clone());
					let name = path.file_name().unwrap().to_string_lossy().into_owned();
					load_levelset(&fs, name, sets);
				}
				// Check for packed files if no directory by that name exist
				else if path.is_file() && !path.with_extension("").exists() {
					let Some(ext) = path.extension() else { continue; };
					if ext == "paks" {
						let key_name = format!("chipgame_{}", path.file_stem().unwrap().to_string_lossy());
						let key = match std::env::var(key_name) {
							Ok(val) => {
								match paks::parse_key(&val) {
								Ok(key) => key,
								Err(err) => {
									eprintln!("Invalid key: {err}: {}", path.display());
									continue;
								}
							}},
							Err(_) => paks::Key::default(),
						};
						match paks::FileReader::open(&path, &key) {
							Ok(paks) => {
								let fs = FileSystem::Paks(paks, key);
								let name = path.file_stem().unwrap().to_string_lossy().into_owned();
								load_levelset(&fs, name, sets);
							},
							Err(err) => {
								eprintln!("Error reading {}: {}", path.display(), err);
							}
						};
					}
					else if ext == "dat" {
						load_dat(&path, sets);
					}
				}
			}
			Err(err) => {
				eprintln!("Error reading set: {}", err);
			}
		}
	}
	sets.sort_by(|a, b| a.name.cmp(&b.name));
}

fn load_levelset(fs: &FileSystem, name: String, sets: &mut Vec<LevelSet>) {
	let index: LevelSetDto = {
		let index = match fs.read_compressed("index.json") {
			Ok(data) => data,
			Err(err) => {
				eprintln!("Error reading index.json: {}", err);
				return;
			}
		};
		match serde_json::from_slice(&index) {
			Ok(level_set) => level_set,
			Err(err) => {
				eprintln!("Error parsing index.json: {}", err);
				return;
			}
		}
	};

	let mut levels = Vec::new();
	for level_ref in index.levels {
		let level = match level_ref {
			LevelRef::Direct(level) => level,
			LevelRef::Indirect(level_path) => {
				let content = match fs.read_to_string(&level_path) {
					Ok(data) => data,
					Err(err) => {
						eprintln!("Error reading {level_path}: {err}");
						continue;
					}
				};
				match serde_json::from_str(&content) {
					Ok(level) => level,
					Err(err) => {
						eprintln!("Error parsing level at {level_path}: {err}");
						continue;
					}
				}
			}
		};
		levels.push(level);
	}

	let splash = index.splash.and_then(|s| match fs {
		FileSystem::StdFs(path) => fs::read(path.join(s)).ok(),
		FileSystem::Paks(paks, key) => paks.read(s.as_bytes(), key).ok(),
	});

	let title = index.title;
	let about = index.about.map(|lines| lines.join("\n"));
	sets.push(LevelSet { name, title, about, splash, levels });
}

fn load_dat(path: &PathBuf, sets: &mut Vec<LevelSet>) {
	let opts = chipdat::Options {
		encoding: chipdat::Encoding::Windows1252,
	};
	let dat = match chipdat::read(path, &opts) {
		Ok(dat) => dat,
		Err(err) => {
			eprintln!("Error reading {}: {:?}", path.display(), err);
			return;
		}
	};

	let name = path.file_stem().map(|s| String::from_utf8_lossy(s.as_encoded_bytes()).into_owned()).unwrap_or(String::new());
	let title = format!("{}", name);

	let mut levels = Vec::new();
	for level in &dat.levels {
		let (map, ents, mut conns) = dat::parse_content(&level.top_layer, &level.bottom_layer);

		if let Some(traps) = &level.metadata.traps {
			for lnk in traps {
				conns.push(chipty::FieldConn {
					src: cvmath::Vec2i(lnk.brown_button_x as i32, lnk.brown_button_y as i32),
					dest: cvmath::Vec2i(lnk.trap_x as i32, lnk.trap_y as i32),
				});
			}
		}

		if let Some(cloners) = &level.metadata.cloners {
			for lnk in cloners {
				conns.push(chipty::FieldConn {
					src: cvmath::Vec2i(lnk.red_button_x as i32, lnk.red_button_y as i32),
					dest: cvmath::Vec2i(lnk.cloner_x as i32, lnk.cloner_y as i32),
				});
			}
		}

		let mut level = chipty::LevelDto {
			name: level.metadata.title.clone().unwrap(),
			author: level.metadata.author.clone(),
			hint: level.metadata.hint.clone(),
			password: level.metadata.password.clone(),
			time_limit: level.time_limit as i32,
			required_chips: level.required_chips as i32,
			map,
			entities: ents,
			connections: conns,
			replays: None,
			trophies: None,
		};

		dat::post_process(&mut level);

		levels.push(level);
	}

	sets.push(LevelSet { name, title, about: None, splash: None, levels });
}

mod dat {
	use chipty::*;
	use std::collections::HashMap;

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

			0x20 => terrain[index] = Terrain::Blank, // Used internally as the Combination tile.
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
			0x33 => terrain[index] = Terrain::WaterHazard, // Drowned Chip
			0x34 => terrain[index] = Terrain::Blank, // Burned Chip
			0x35 => terrain[index] = Terrain::Blank, // Burned Chip
			0x36 => terrain[index] = Terrain::InvisibleWall, // This byte does not correspond to any defined tile, but acts like an invisible wall.
			0x37 => terrain[index] = Terrain::InvisibleWall, // This byte does not correspond to any defined tile, but acts like an invisible wall.
			0x38 => entities.push(ent_args(EntityKind::IceBlock, pos, None)),
			0x39 => terrain[index] = Terrain::FakeExit,
			0x3a => terrain[index] = Terrain::FakeExit,
			0x3b => terrain[index] = Terrain::FakeExit,
			0x3c => terrain[index] = Terrain::WaterHazard, // Swimming Chip (N)
			0x3d => terrain[index] = Terrain::WaterHazard, // Swimming Chip (W)
			0x3e => terrain[index] = Terrain::WaterHazard, // Swimming Chip (S)
			0x3f => terrain[index] = Terrain::WaterHazard, // Swimming Chip (E)

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

			0x70 => terrain[index] = Terrain::Blank, // Values of $70 and above are used internally for graphical transparency.
			value => unimplemented!("Tile: ${:02x}", value),
		}
	}

	pub fn parse_content(upper: &[u8], lower: &[u8]) -> (FieldDto, Vec<EntityArgs>, Vec<FieldConn>) {
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

		// Replace Block entites on Floor with DirtBlock terrain
		entities.retain_mut(|ent| {
			let index = (ent.pos.y * 32 + ent.pos.x) as usize;
			if matches!(ent.kind, EntityKind::Block) && matches!(terrain[index], Terrain::Floor) {
				terrain[index] = Terrain::DirtBlock;
				return false;
			}
			true
		});

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
						conns.push(FieldConn { src: prev_teleport, dest: pos });
					}
					prev_teleport = Some(pos);
				}
			}
		}
		if let Some(last_teleport) = last_teleport {
			if let Some(prev_teleport) = prev_teleport {
				conns.push(FieldConn { src: prev_teleport, dest: last_teleport });
			}
		}

		let mut legend = Vec::new();
		let data = {
			let mut legend_map = HashMap::new();
			legend_map.insert(Terrain::Blank, 0); legend.push(Terrain::Blank);
			legend_map.insert(Terrain::Floor, 1); legend.push(Terrain::Floor);
			let mut idx = 2;
			for &terrain in terrain.iter() {
				if !legend_map.contains_key(&terrain) {
					legend_map.insert(terrain, idx);
					legend.push(terrain);
					idx += 1;
				}
			}
			terrain.iter().map(|&terrain| legend_map[&terrain]).collect()
		};

		let map = FieldDto { width: 32, height: 32, data, legend };
		return (map, entities, conns);
	}

	fn ent_args(kind: EntityKind, pos: cvmath::Vec2i, face_dir: Option<Compass>) -> EntityArgs {
		EntityArgs { kind, pos, face_dir }
	}

	pub fn post_process(level: &mut LevelDto) -> bool {
		let mut fixed = false;

		let mut ents_to_remove = Vec::new();

		// Replace Block entities targetted by a red connection with CloneBlock terrain
		for (ent_index, ent_args) in level.entities.iter().enumerate() {
			if !matches!(ent_args.kind, EntityKind::Block) {
				continue;
			}

			let Some(&conn) = level.connections.iter().find(|&conn| conn.dest == ent_args.pos) else {
				continue
			};

			{
				let index = (conn.src.y * level.map.width + conn.src.x) as usize;
				let tile = level.map.data[index] as usize;
				let terrain = level.map.legend[tile];
				if !matches!(terrain, Terrain::RedButton) {
					continue;
				}
			}

			let new_terrain = match ent_args.face_dir {
				Some(Compass::Up) => Terrain::CloneBlockN,
				Some(Compass::Down) => Terrain::CloneBlockS,
				Some(Compass::Left) => Terrain::CloneBlockW,
				Some(Compass::Right) => Terrain::CloneBlockE,
				_ => continue,
			};

			ents_to_remove.push(ent_index);

			let new_tile = {
				if let Some(new_tile) = level.map.legend.iter().position(|&t| t == new_terrain) {
					new_tile as u8
				}
				else {
					level.map.legend.push(new_terrain);
					level.map.legend.len() as u8 - 1
				}
			};

			let index = (ent_args.pos.y * level.map.width + ent_args.pos.x) as usize;
			level.map.data[index] = new_tile;
			fixed = true;
		}

		for &ent_index in ents_to_remove.iter().rev() {
			level.entities.remove(ent_index);
		}

		return fixed;
	}
}
