
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ModelId {
	Empty,
	Floor,
	Wall,
	ToggleWall,
	Sprite,
	FlatSprite,
	ReallyFlatSprite,
	FloorSprite,
	EndPortal,
	Tank,
	Glider,
}
