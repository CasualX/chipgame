use std::{fs, io, path};

pub struct Image {
	pub width: i32,
	pub height: i32,
	pub data: Vec<u8>,
}

impl Image {
	pub fn new(width: i32, height: i32, data: Vec<u8>) -> Image {
		assert!(data.len() == (width * height * 4) as usize);
		Image { width, height, data }
	}

	pub fn empty(width: i32, height: i32) -> Image {
		Image::new(width, height, vec![0; (width * height * 4) as usize])
	}

	pub fn load_file(path: &path::Path) -> Image {
		load_frame(path)
	}

	pub fn area(&self, gutter: i32) -> i32 {
		(self.width + gutter * 2) * (self.height + gutter * 2)
	}

	pub fn save(&self, path: &path::Path) {
		save_image(self, path);
	}

	pub fn copy_image(&mut self, src: &Image, offset_x: i32, offset_y: i32, gutter: i32) {
		blit(self, src, offset_x, offset_y, gutter);
	}

	pub fn recover_alpha_colors(&mut self) {
		*self = recover_alpha_colors(self);
	}
}

fn load_frame(path: &path::Path) -> Image {
	let file = fs::File::open(path).expect("sprite png readable");
	let decoder = png::Decoder::new(io::BufReader::new(file));
	let mut reader = decoder.read_info().expect("valid png stream");
	let mut buf = vec![0; reader.output_buffer_size()];
	let info = reader
		.next_frame(&mut buf)
		.expect("able to decode png frame");
	let data = buf[..info.buffer_size()].to_vec();
	let pixels = get_pixels(&info, data);
	Image {
		width: info.width as i32,
		height: info.height as i32,
		data: pixels,
	}
}

fn get_pixels(info: &png::OutputInfo, data: Vec<u8>) -> Vec<u8> {
	// Only support 8-bit Rgba images
	assert_eq!(info.bit_depth, png::BitDepth::Eight);

	if info.color_type == png::ColorType::Rgb {
		// Convert Rgb to Rgba by adding an opaque alpha channel
		let mut rgba_pixels = vec![0; (info.width * info.height * 4) as usize];
		for i in 0..(info.width * info.height) as usize {
			rgba_pixels[i * 4] = data[i * 3];
			rgba_pixels[i * 4 + 1] = data[i * 3 + 1];
			rgba_pixels[i * 4 + 2] = data[i * 3 + 2];
			rgba_pixels[i * 4 + 3] = 255; // Opaque alpha
		}
		return rgba_pixels;
	}
	else {
		assert_eq!(info.color_type, png::ColorType::Rgba);
		return data;
	}
}

fn save_image(image: &Image, path: &path::Path) {
	assert_eq!(
		image.data.len(),
		(image.width * image.height * 4) as usize,
		"image buffer matches dimensions"
	);
	let file = fs::File::create(path).expect("create output spritesheet");
	let writer = io::BufWriter::new(file);
	let mut encoder = png::Encoder::new(writer, image.width as u32, image.height as u32);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	let mut png_writer = encoder.write_header().expect("write png header");
	png_writer
		.write_image_data(&image.data)
		.expect("write png image data");
}

fn blit(dest: &mut Image, src: &Image, offset_x: i32, offset_y: i32, gutter: i32) {
	let start_x = (offset_x - gutter).max(0);
	let start_y = (offset_y - gutter).max(0);
	let end_x = (offset_x + src.width + gutter).min(dest.width);
	let end_y = (offset_y + src.height + gutter).min(dest.height);
	if start_x >= end_x || start_y >= end_y {
		return;
	}

	let src_stride = src.width as usize * 4;
	let dest_stride = dest.width as usize * 4;

	for y in start_y..end_y {
		let src_y = i32::clamp(y - offset_y, 0, src.height - 1) as usize;
		let dest_row = y as usize * dest_stride;
		let src_row = src_y * src_stride;
		for x in start_x..end_x {
			let src_x = i32::clamp(x - offset_x, 0, src.width - 1) as usize;
			let dest_idx = dest_row + x as usize * 4;
			let src_idx = src_row + src_x * 4;
			dest.data[dest_idx..dest_idx + 4].copy_from_slice(&src.data[src_idx..src_idx + 4]);
		}
	}
}

fn recover_alpha_colors(image: &Image) -> Image {
	let width = image.width;
	let height = image.height;
	let src = &image.data;
	let mut dest = vec![0; (width * height * 4) as usize];

	// Copy the pixels but if the src is transparent (alpha == 0)
	// Then take the average of the surrounding (non transparent) pixels
	for y in 0..height {
		for x in 0..width {
			let idx = ((y * width + x) * 4) as usize;
			let alpha = src[idx + 3];

			if alpha == 0 {
				let mut r_sum = 0u32;
				let mut g_sum = 0u32;
				let mut b_sum = 0u32;
				let mut count = 0;

				for dy in -1..=1 {
					for dx in -1..=1 {
						if dx == 0 && dy == 0 {
							continue; // Skip the current pixel
						}

						let nx = x + dx;
						let ny = y + dy;

						if nx >= 0 && nx < width && ny >= 0 && ny < height {
							let n_idx = ((ny * width + nx) * 4) as usize;
							let n_alpha = src[n_idx + 3];

							if n_alpha == 255 {
								r_sum += src[n_idx] as u32;
								g_sum += src[n_idx + 1] as u32;
								b_sum += src[n_idx + 2] as u32;
								count += 1;
							}
						}
					}
				}

				if count > 0 {
					dest[idx] = (r_sum / count) as u8;
					dest[idx + 1] = (g_sum / count) as u8;
					dest[idx + 2] = (b_sum / count) as u8;
				}
				else {
					dest[idx] = 0;
					dest[idx + 1] = 0;
					dest[idx + 2] = 0;
				}
				dest[idx + 3] = 0; // Keep the alpha channel transparent
			}
			else {
				// Copy original pixel
				dest[idx] = src[idx];
				dest[idx + 1] = src[idx + 1];
				dest[idx + 2] = src[idx + 2];
				dest[idx + 3] = alpha;
			}
		}
	}

	Image {
		width,
		height,
		data: dest,
	}
}
