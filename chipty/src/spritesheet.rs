use super::*;

fn default_len() -> u16 { 1 }
fn is_default_len(len: &u16) -> bool { *len == default_len() }

/// An animated sprite description.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct SpriteEntry {
	/// The index of the first frame.
	pub index: u16,
	/// The number of frames in the animation.
	#[serde(default = "default_len", skip_serializing_if = "is_default_len")]
	pub len: u16,
	/// The duration of the animation in seconds.
	#[serde(skip)] // Set to the sum of the durations of the frames when loaded.
	pub duration: f32,
}

/// A single frame in the sprite sheet.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct SpriteFrame {
	/// The location of the sprite in the image
	///
	/// `[x, y, width, height]` in pixels.
	pub rect: [i32; 4],
	/// The origin of the sprite in pixels, relative to the top-left corner of the rect.
	#[serde(default, skip_serializing_if = "is_default")]
	pub origin: Vec2<i32>,
	/// Frame duration in seconds.
	#[serde(default, skip_serializing_if = "is_default")]
	pub duration: f32,
}

/// A collection of sprites in a spritesheet image.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Default, Debug)]
pub struct SpriteSheet<T> {
	/// The width of the sheet in pixels.
	pub width: i32,
	/// The height of the sheet in pixels.
	pub height: i32,
	/// The sprites in the sheet.
	#[serde(bound(deserialize = "BTreeMap<T, SpriteEntry>: serde::Deserialize<'de>"))]
	pub sprites: BTreeMap<T, SpriteEntry>,
	/// The frame data referenced by the sprites.
	pub frames: Vec<SpriteFrame>,
}
