
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
}

impl ModelId {
	#[inline]
	pub fn is_solid(self) -> bool {
		matches!(self, ModelId::Wall | ModelId::ToggleWall)
	}
}
