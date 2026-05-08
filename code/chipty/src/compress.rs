use std::io::prelude::*;

pub fn compress(data: &[u8]) -> Vec<u8> {
	let mut z = flate2::bufread::ZlibEncoder::new(data, flate2::Compression::best());
	let mut buf = Vec::new();
	z.read_to_end(&mut buf).unwrap();
	buf
}

pub fn decompress(data: &[u8]) -> Vec<u8> {
	let mut z = flate2::bufread::ZlibDecoder::new(data);
	let mut buf = Vec::new();
	z.read_to_end(&mut buf).unwrap();
	buf
}

pub fn encode(bytes: &[u8]) -> String {
	// Compress the bytes
	let compressed = compress(bytes);
	// Base64 encode to string
	basenc::Base64Std.encode(&compressed)
}

pub fn decode(string: &str) -> Vec<u8> {
	// Base64 decode to bytes
	let data = basenc::Base64Std.decode(string).unwrap();
	// Decompress the bytes
	decompress(&data)
}
