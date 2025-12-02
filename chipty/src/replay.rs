// Replay data transfer object.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ReplayDto {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub date: Option<String>,
	pub ticks: i32,
	pub realtime: f32,
	pub steps: i32,
	pub bonks: i32,
	pub seed: String,
	pub replay: String,
}
