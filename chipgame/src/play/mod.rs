//! Gameplay module.

use std::{io, mem, path, fs};

use super::*;

mod event;
mod playstate;
mod savedata;
mod lvsets;
mod tiles;

pub use self::event::*;
pub use self::playstate::*;
pub use self::savedata::*;
pub use self::lvsets::*;
pub use self::tiles::*;

#[cfg(not(target_arch = "wasm32"))]
fn write_file(path: &path::Path, content: &str) -> io::Result<()> {
	if let Some(parent) = path.parent() {
		let _ = fs::create_dir_all(parent);
	}
	fs::write(path, content)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file(path: &path::Path) -> io::Result<String> {
	fs::read_to_string(path)
}

#[cfg(target_arch = "wasm32")]
fn write_file(path: &path::Path, content: &str) -> io::Result<()> {
	extern "C" {
		fn chipgame_write_file(path_ptr: *const u8, path_len: usize, content_ptr: *const u8, content_len: usize) -> i32;
	}
	let path = path.as_os_str().as_encoded_bytes();
	let content = content.as_bytes();
	unsafe {
		let result = chipgame_write_file(path.as_ptr(), path.len(), content.as_ptr(), content.len());
		if result == 0 {
			Ok(())
		}
		else {
			Err(io::Error::new(io::ErrorKind::Other, "Failed to write file"))
		}
	}
}

#[cfg(target_arch = "wasm32")]
fn read_file(path: &path::Path) -> io::Result<String> {
	#[allow(improper_ctypes)]
	extern "C" {
		fn chipgame_read_file(path_ptr: *const u8, path_len: usize, content_ptr: *mut String) -> i32;
	}
	let path = path.as_os_str().as_encoded_bytes();
	unsafe {
		let mut content = String::new();
		let result = chipgame_read_file(path.as_ptr(), path.len(), &mut content as *mut String);
		if result == 0 {
			Ok(content)
		}
		else {
			Err(io::Error::new(io::ErrorKind::Other, "Failed to read file"))
		}
	}
}
