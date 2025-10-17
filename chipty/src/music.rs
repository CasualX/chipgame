
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MusicId {
	MenuMusic,
	GameMusic,
}

impl std::str::FromStr for MusicId {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"MenuMusic" => Ok(MusicId::MenuMusic),
			"GameMusic" => Ok(MusicId::GameMusic),
			_ => Err(()),
		}
	}
}
