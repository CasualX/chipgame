use std::path::PathBuf;

use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelSetDto {
	pub name: String,
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub about: Option<Vec<String>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub splash: Option<String>,
	#[serde(default)]
	pub unlock_all_levels: bool,
	pub levels: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelData {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hint: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub password: Option<String>,
}

#[derive(Default)]
pub struct LevelSet {
	pub name: String,
	pub title: String,
	pub about: Option<String>,
	pub splash: Option<PathBuf>,
	pub unlock_all_levels: bool,
	pub lv_data: Vec<String>,
	pub lv_info: Vec<LevelData>,
}
impl LevelSet {
	pub fn get_level_number(&self, name: &str) -> Option<i32> {
		self.lv_info.iter().position(|s| s.name == name).map(|i| i as i32 + 1)
	}
	pub fn get_level_index(&self, name: &str) -> Option<usize> {
		self.lv_info.iter().position(|s| s.name == name)
	}
}

#[derive(Default)]
pub struct LevelSets {
	pub selected: usize,
	pub collection: Vec<LevelSet>,
}
impl LevelSets {
	pub fn current(&self) -> &LevelSet {
		&self.collection[self.selected]
	}
	pub fn load(&mut self) {
		load_levelsets(&mut self.collection);
	}
}

fn load_levelsets(packs: &mut Vec<LevelSet>) {
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
					load_levelset_pak(&fs, packs);
				}
				// Check for .paks files and if a folder by that name does not exist
				if path.is_file() && path.extension().map_or(false, |ext| ext == "paks") && !path.with_extension("").exists() {
					match paks::FileReader::open(&path, &paks::Key::default()) {
						Ok(paks) => {
							let fs = FileSystem::Paks(paks);
							load_levelset_pak(&fs, packs);
						},
						Err(err) => {
							eprintln!("Error reading {}: {}", path.display(), err);
						}
					};
				}
			}
			Err(err) => {
				eprintln!("Error reading pack: {}", err);
			}
		}
	}
	packs.sort_by(|a, b| a.name.cmp(&b.name));
}

fn load_levelset_pak(fs: &FileSystem, packs: &mut Vec<LevelSet>) {
	let index: LevelSetDto = {
		let index = match fs.read("index.json") {
			Ok(data) => data,
			Err(err) => {
				eprintln!("Error reading index.json: {}", err);
				return;
			}
		};
		match serde_json::from_slice(&index) {
			Ok(pack) => pack,
			Err(err) => {
				eprintln!("Error parsing index.json: {}", err);
				return;
			}
		}
	};

	let mut lv_info = Vec::new();
	let mut lv_data = Vec::new();
	for level_path in &index.levels {
		let s = match fs.read_to_string(level_path) {
			Ok(data) => data,
			Err(err) => {
				eprintln!("Error reading {level_path}: {}", err);
				continue;
			}
		};

		let ld: LevelData = match serde_json::from_str(&s) {
			Ok(ld) => ld,
			Err(err) => {
				eprintln!("Error parsing {level_path}: {}", err);
				continue;
			}
		};

		lv_info.push(ld);
		lv_data.push(s);
	}

	let splash = index.splash.map(|s| match fs {
		FileSystem::StdFs(path) => path.join(s),
		FileSystem::Paks(_) => PathBuf::from(s),// This is wrong, load the splash image here... Or pass the FS through everywhere
	});

	packs.push(LevelSet {
		name: index.name,
		title: index.title,
		about: index.about.map(|lines| lines.join("\n")),
		splash,
		unlock_all_levels: index.unlock_all_levels,
		lv_data,
		lv_info,
	});
}
