use super::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LevelSetDto {
	pub name: String,
	pub title: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub about: Option<Vec<String>>,
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
					load_levelset(&path, packs);
				}
			}
			Err(err) => {
				eprintln!("Error reading pack: {}", err);
			}
		}
	}
	packs.sort_by(|a, b| a.name.cmp(&b.name));
}

fn load_levelset(path: &path::Path, packs: &mut Vec<LevelSet>) {
	let index_path = path.join("index.json");
	let json = match fs::read_to_string(&index_path) {
		Ok(json) => json,
		Err(err) => {
			eprintln!("Error reading {}: {}", index_path.display(), err);
			return;
		}
	};

	let pack: LevelSetDto = match serde_json::from_str(&json) {
		Ok(pack) => pack,
		Err(err) => {
			eprintln!("Error parsing {}: {}", index_path.display(), err);
			return;
		}
	};

	let mut lv_info = Vec::new();
	let mut lv_data = Vec::new();
	for level in &pack.levels {
		let level_path = path.join(level);
		let s = match fs::read_to_string(&level_path) {
			Ok(s) => s,
			Err(err) => {
				eprintln!("Error reading {}: {}", level_path.display(), err);
				continue;
			}
		};

		let ld: LevelData = match serde_json::from_str(&s) {
			Ok(ld) => ld,
			Err(err) => {
				eprintln!("Error parsing {}: {}", level_path.display(), err);
				continue;
			}
		};

		lv_info.push(ld);
		lv_data.push(s);
	}

	packs.push(LevelSet {
		name: pack.name,
		title: pack.title,
		about: pack.about.map(|lines| lines.join("\n")),
		unlock_all_levels: pack.unlock_all_levels,
		lv_data,
		lv_info,
	});
}
