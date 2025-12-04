use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::{fs, io, path};

use chipty::{SpriteEntry, SpriteFrame, SpriteSheet};
use cvmath::Vec2;

mod binpack;
mod image;

use binpack::GridBinPacker;
use image::Image;

struct Sprite {
	name: String,
	frames: Vec<Image>,
}

const SPRITE_FRAME_TIME: f32 = 0.0; // FIXME: set proper frame time

const GUTTER: i32 = 2;

enum SpriteSize {
	// 32x32
	Square,
	// 64x32
	Wide,
	// 32x64
	Tall,
}

fn sprite_size(image: &Image) -> Option<SpriteSize> {
	if image.width == 32 && image.height == 32 {
		Some(SpriteSize::Square)
	}
	else if image.width == 64 && image.height == 32 {
		Some(SpriteSize::Wide)
	}
	else if image.width == 32 && image.height == 64 {
		Some(SpriteSize::Tall)
	}
	else {
		None
	}
}

fn sprite_area(sprite: &Sprite) -> i32 {
	sprite.frames.iter().map(|frame| frame.area(GUTTER)).sum()
}

fn main() {
	let sprites = load_sprites(path::Path::new("gfx/MS/"));
	println!("Compiled {} sprites", sprites.len());
	let mut total_area = 0;
	for sprite in &sprites {
		total_area += sprite_area(sprite);
		if let Some(first) = sprite.frames.first() {
			let total_bytes: usize = sprite.frames.iter().map(|frame| frame.data.len()).sum();
			println!(
				"{} -> {} frame(s), {}x{}, {} bytes",
				sprite.name,
				sprite.frames.len(),
				first.width,
				first.height,
				total_bytes
			);
		}
	}
	println!("Total sprite area: {} pixels", total_area);
	let sheet_width = 1024;
	let sheet_height = 512;
	assert!(total_area <= sheet_width * sheet_height);

	let mut sheet = Image::empty(sheet_width, sheet_height);

	let mut packer = GridBinPacker::new(sheet_width, sheet_height, 1);
	let mut packing_order: Vec<usize> = (0..sprites.len()).collect();
	packing_order.sort_by_key(|&idx| Reverse(sprite_area(&sprites[idx])));
	let mut packed_frames = 0;
	let mut frames_meta: Vec<SpriteFrame> = Vec::new();
	let mut sprite_entries: BTreeMap<String, SpriteEntry> = BTreeMap::new();
	for idx in packing_order {
		let sprite = &sprites[idx];
		let sprite_start = frames_meta.len();
		for (frame_idx, frame) in sprite.frames.iter().enumerate() {
			sprite_size(frame).expect("sprites fit expected tile sizes");
			let padded_width = frame.width + GUTTER * 2;
			let padded_height = frame.height + GUTTER * 2;
			let (x, y) = packer
				.insert(padded_width, padded_height)
				.unwrap_or_else(|| panic!("sheet too small for {} frame {}", sprite.name, frame_idx));
			let draw_x = x + GUTTER;
			let draw_y = y + GUTTER;
			sheet.copy_image(frame, draw_x, draw_y, GUTTER);
			frames_meta.push(SpriteFrame {
				rect: [draw_x, draw_y, frame.width, frame.height],
				origin: Vec2::new(0, 0),
				duration: SPRITE_FRAME_TIME,
			});
			packed_frames += 1;
		}
		let sprite_len = frames_meta.len() - sprite_start;
		let entry = SpriteEntry {
			index: sprite_start
				.try_into()
				.expect("frame index fits in u16"),
			len: sprite_len
				.try_into()
				.expect("frame count fits in u16"),
			duration: SPRITE_FRAME_TIME * sprite_len as f32,
		};
		sprite_entries.insert(sprite.name.clone(), entry);
	}

	sheet.recover_alpha_colors();
	sheet.save(path::Path::new("data/spritesheet.png"));
	let sheet_meta = SpriteSheet {
		width: sheet_width,
		height: sheet_height,
		sprites: sprite_entries,
		frames: frames_meta,
	};
	save_metadata(&sheet_meta, path::Path::new("data/spritesheet.json"));
	println!(
		"Packed {} frames from {} sprites",
		packed_frames,
		sprites.len()
	);
}

fn load_sprites(root: &path::Path) -> Vec<Sprite> {
	let mut grouped: BTreeMap<(String, bool), Vec<(usize, Image)>> = BTreeMap::new();

	for entry in fs::read_dir(root).expect("sprites directory accessible") {
		let entry = entry.expect("valid dir entry");
		let path = entry.path();
		if !path.is_file() {
			continue;
		}
		if !is_png(&path) {
			continue;
		}

		let stem = path
			.file_stem()
			.and_then(|s| s.to_str())
			.expect("sprite filename has valid stem");
		let (base_name, frame_idx) = parse_sprite_name(stem);
		let frame = Image::load_file(&path);
		let key = (base_name, frame_idx.is_some());
		grouped
			.entry(key)
			.or_default()
			.push((frame_idx.unwrap_or(1), frame));
	}

	let mut sprites: Vec<Sprite> = grouped
		.into_iter()
		.map(|((mut name, animated), mut frames)| {
			frames.sort_by_key(|(idx, _)| *idx);
			let frames = frames.into_iter().map(|(_, frame)| frame).collect();
			if animated {
				name.push('A');
			}
			Sprite { name, frames }
		})
		.collect();

	sprites.sort_by(|a, b| a.name.cmp(&b.name));
	sprites
}

fn is_png(path: &path::Path) -> bool {
	path
		.extension()
		.and_then(|ext| ext.to_str())
		.map(|ext| ext.eq_ignore_ascii_case("png"))
		.unwrap_or(false)
}

fn parse_sprite_name(stem: &str) -> (String, Option<usize>) {
	if let Some((base, suffix)) = stem.rsplit_once('_') {
		if suffix.chars().all(|c| c.is_ascii_digit()) {
			return (base.to_string(), Some(suffix.parse().expect("numeric frame index")));
		}
	}
	(stem.to_string(), None)
}

fn save_metadata(sheet: &SpriteSheet<String>, path: &path::Path) {
	let file = fs::File::create(path).expect("create spritesheet metadata json");
	let writer = io::BufWriter::new(file);
	serde_json::to_writer(writer, sheet).expect("serialize spritesheet metadata");
}
