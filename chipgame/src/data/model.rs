
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ModelId {
	Empty,
	Floor,
	Wall,
	ThinWall,
	Sprite,
	FlatSprite,
	ReallyFlatSprite,
	FloorSprite,
	Portal,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnimationId {
	None,
	FadeOut,
	FadeIn,
	Fall,
	Raise,
}
