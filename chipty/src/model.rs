
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
}
