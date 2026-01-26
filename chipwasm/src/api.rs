
extern "C" {
	fn randomBytes(ptr: *mut u8, len: usize);
	pub fn play_sound(sound_ptr: *const u8, sound_len: usize);
	pub fn play_music(music_ptr: *const u8, music_len: usize);
	pub fn set_title(title_ptr: *const u8, title_len: usize);
	pub fn quit_game();
}

// Support getrandom in wasm builds

#[no_mangle]
unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), getrandom_03::Error> {
	unsafe { randomBytes(dest, len) };
	Ok(())
}

fn __getrandom_02_custom(buf: &mut [u8]) -> Result<(), getrandom_02::Error> {
	unsafe { randomBytes(buf.as_mut_ptr(), buf.len()) };
	Ok(())
}

getrandom_02::register_custom_getrandom!(__getrandom_02_custom);
