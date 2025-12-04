use std::fs;
use std::path::PathBuf;

fn main() {
	let matches = clap::command!()
		.arg(clap::arg!(-i <input> "Input image file").value_parser(clap::value_parser!(PathBuf)))
		.arg(clap::arg!(-o [output] "Output image file").value_parser(clap::value_parser!(PathBuf)))
		.get_matches();

	let input_file = matches.get_one::<PathBuf>("input").expect("Missing input file argument").clone();
	let output_file = matches.get_one::<PathBuf>("output").cloned().unwrap_or_else(|| input_file.clone());

	// Load the image file and replace pink with transparent color
	let mut decoder = png::Decoder::new(fs::File::open(&input_file).unwrap());
	decoder.set_transformations(png::Transformations::normalize_to_color8());
	let mut reader = decoder.read_info().unwrap();
	let mut pixels = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut pixels).unwrap();

	// Only support 8-bit Rgba images
	assert_eq!(info.bit_depth, png::BitDepth::Eight);

	if info.color_type == png::ColorType::Rgb {
		// Convert Rgb to Rgba by adding an opaque alpha channel
		let mut rgba_pixels = vec![0; (info.width * info.height * 4) as usize];
		for i in 0..(info.width * info.height) as usize {
			rgba_pixels[i * 4] = pixels[i * 3];
			rgba_pixels[i * 4 + 1] = pixels[i * 3 + 1];
			rgba_pixels[i * 4 + 2] = pixels[i * 3 + 2];
			rgba_pixels[i * 4 + 3] = 255; // Opaque alpha
		}
		pixels = rgba_pixels;
	}
	else {
		assert_eq!(info.color_type, png::ColorType::Rgba);
	}

	let width = info.width as i32;
	let height = info.height as i32;
	let mut image = vec![0; (width * height * 4) as usize];

	copy_pixels(
		&pixels, info.width as i32, info.height as i32,
		&mut image, width, height,
		0, 0, 0, 0, width, height);

	copy_pixels(
		&pixels, info.width as i32, info.height as i32,
		&mut image, width, height,
		0, 0, 0, 0, width, height);

	copy_pixels(
		&pixels, info.width as i32, info.height as i32,
		&mut image, width, height,
		0, 0, 0, 0, width, height);

	let mut buffer = vec![0; image.len()];
	recover_alpha_colors(&image, &mut buffer, width, height);

	let mut encoder = png::Encoder::new(fs::File::create(&output_file).unwrap(), width as u32, height as u32);
	encoder.set_color(png::ColorType::Rgba);
	encoder.set_depth(png::BitDepth::Eight);
	encoder.set_compression(png::Compression::Best);
	let mut writer = encoder.write_header().unwrap();
	writer.write_image_data(&buffer).unwrap();
	println!("Converted {} to {}", input_file.display(), output_file.display());
}

fn copy_pixels(
	input_pixels: &[u8],
	input_width: i32,
	input_height: i32,
	output_pixels: &mut [u8],
	output_width: i32,
	output_height: i32,
	src_x: i32,
	src_y: i32,
	dst_x: i32,
	dst_y: i32,
	width: i32,
	height: i32,
) {
	for y in 0..height {
		for x in 0..width {
			let src_index = (((src_y + y) * input_width + (src_x + x)) * 4) as usize;
			let dst_index = (((dst_y + y) * output_width + (dst_x + x)) * 4) as usize;
			if input_pixels[src_index] == 255 && input_pixels[src_index + 1] == 0 && input_pixels[src_index + 2] == 255 {
				// Replace pink with transparent
				output_pixels[dst_index] = 0;
				output_pixels[dst_index + 1] = 0;
				output_pixels[dst_index + 2] = 0;
				output_pixels[dst_index + 3] = 0;
			}
			else {
				// Copy the pixel as is
				output_pixels[dst_index..dst_index + 4].copy_from_slice(&input_pixels[src_index..src_index + 4]);
			}
		}
	}
}

fn recover_alpha_colors(src: &Vec<u8>, dest: &mut Vec<u8>, width: i32, height: i32) {
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

}
