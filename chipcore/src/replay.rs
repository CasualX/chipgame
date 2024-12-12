use std::io::prelude::*;


#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Debug, Default)]
pub struct ReplayDto {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub date: Option<String>,
	pub ticks: i32,
	pub realtime: f32,
	pub steps: i32,
	pub bonks: i32,
	pub seed: String,
	pub replay: String,
}

pub(crate) fn encode_bytes(bytes: &[u8]) -> String {
	// Compress the bytes
	let mut z = flate2::bufread::ZlibEncoder::new(bytes, flate2::Compression::best());
	let mut buf = Vec::new();
	z.read_to_end(&mut buf).unwrap();

	// Base64 encode to string
	basenc::Base64Std.encode(&buf)
}

pub fn decode_bytes(string: &str) -> Vec<u8> {
	// Base64 decode to bytes
	let data = basenc::Base64Std.decode(string).unwrap();

	// Decompress the bytes
	let mut z = flate2::bufread::ZlibDecoder::new(&data[..]);
	let mut buf = Vec::new();
	z.read_to_end(&mut buf).unwrap();

	return buf;
}
