use super::*;

// Replay data transfer object.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ReplayDto {
	/// Optional level name.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub level_name: Option<String>,
	/// Optional date string.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub date: Option<String>,
	/// Total number of attempts.
	#[serde(default)]
	pub attempts: i32,
	/// Real time taken in seconds.
	pub realtime: f32,
	/// In-game ticks taken to reach LevelComplete state.
	pub ticks: i32,
	/// Number of steps taken.
	pub steps: i32,
	/// Number of bonks.
	pub bonks: i32,
	/// RNG seed used.
	pub seed: String,
	/// Encoded inputs.
	///
	/// Number of inputs is allowed to not match ticks exactly, as long as LevelComplete is reached.
	///
	/// Decode and decompress with [decode].
	#[serde(alias = "replay")]
	pub inputs: String,
	/// Number of times the gameplay was unpaused.
	#[serde(default, skip_serializing_if = "is_default")]
	pub unpauses: i32,
	/// Number of warps set.
	#[serde(default, skip_serializing_if = "is_default")]
	pub warps_set: i32,
	/// Number of warps actually used.
	#[serde(default, skip_serializing_if = "is_default")]
	pub warps_used: i32,
}
