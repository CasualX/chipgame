
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ModelId {
	Empty,
	Floor,
	Wall,
	ThinWall,
	WallV2,
	Sprite,
	SpriteShadow,
	FlatSprite,
	ReallyFlatSprite,
	FloorSprite,
	Portal,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnimationId {
	None,
	Rise,
	FadeOut,
	FadeIn,
	Fall,
	Raise,
}
