use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::{fs, io, path};

use chipty::{SpriteEntry, SpriteFrame, SpriteSheet};
use cvmath::Vec2;

mod binpack;
mod config;
mod image;

use binpack::GridBinPacker;
use image::Image;

struct FrameAsset {
	file: String,
	image: Image,
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

fn main() {
	let root = path::Path::new("gfx/MS/");
	let sprite_config = config::File::load(root).sprites;
	let frame_assets = load_unique_frames(root, &sprite_config);
	println!(
		"Loaded {} sprite definitions referencing {} unique files",
		sprite_config.len(),
		frame_assets.len()
	);
	let mut total_area = 0;
	for frame in &frame_assets {
		total_area += frame.image.area(GUTTER);
		println!(
			"{} -> {}x{}, {} bytes",
			frame.file,
			frame.image.width,
			frame.image.height,
			frame.image.data.len()
		);
	}
	println!("Total sprite area: {} pixels", total_area);
	let sheet_width = 1024;
	let sheet_height = 512;
	assert!(total_area <= sheet_width * sheet_height);

	let mut sheet = Image::empty(sheet_width, sheet_height);

	let mut packer = GridBinPacker::new(sheet_width, sheet_height, 1);
	let mut packing_order: Vec<usize> = (0..frame_assets.len()).collect();
	packing_order.sort_by_key(|&idx| Reverse(frame_assets[idx].image.area(GUTTER)));
	let mut packed_frames = 0;
	let mut frame_lookup: HashMap<String, SpriteFrame> = HashMap::new();
	for idx in packing_order {
		let frame = &frame_assets[idx];
		sprite_size(&frame.image).expect("sprites fit expected tile sizes");
		let padded_width = frame.image.width + GUTTER * 2;
		let padded_height = frame.image.height + GUTTER * 2;
		let (x, y) = packer
			.insert(padded_width, padded_height)
			.unwrap_or_else(|| panic!("sheet too small for {}", frame.file));
		let draw_x = x + GUTTER;
		let draw_y = y + GUTTER;
		sheet.copy_image(&frame.image, draw_x, draw_y, GUTTER);
		let entry = SpriteFrame {
			rect: [draw_x, draw_y, frame.image.width, frame.image.height],
			origin: Vec2::new(0, 0),
			duration: SPRITE_FRAME_TIME,
		};
		frame_lookup.insert(frame.file.clone(), entry);
		packed_frames += 1;
	}

	let mut frames_meta: Vec<SpriteFrame> = Vec::new();
	let mut sprite_entries: BTreeMap<String, SpriteEntry> = BTreeMap::new();
	for sprite in &sprite_config {
		let sprite_start = frames_meta.len();
		for path in &sprite.frames {
			let meta = frame_lookup
				.get(path)
				.unwrap_or_else(|| panic!("frame {} missing from packed sheet", path));
			frames_meta.push(*meta);
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
	let emitted_frames = frames_meta.len();
	let sheet_meta = SpriteSheet {
		width: sheet_width,
		height: sheet_height,
		sprites: sprite_entries,
		frames: frames_meta,
	};
	save_metadata(&sheet_meta, path::Path::new("data/spritesheet.json"));
	println!(
		"Packed {} unique images, emitted {} sprite frames across {} sprites",
		packed_frames,
		emitted_frames,
		sprite_config.len()
	);
}


fn load_unique_frames(root: &path::Path, sprites: &[config::Sprite]) -> Vec<FrameAsset> {
	let mut seen: HashSet<String> = HashSet::new();
	let mut frames = Vec::new();
	for sprite in sprites {
		for rel in &sprite.frames {
			if seen.insert(rel.clone()) {
				let frame_path = root.join(rel);
				frames.push(FrameAsset {
					file: rel.clone(),
					image: Image::load_file(&frame_path),
				});
			}
		}
	}
	frames
}

fn save_metadata(sheet: &SpriteSheet<String>, path: &path::Path) {
	let file = fs::File::create(path).expect("create spritesheet metadata json");
	let writer = io::BufWriter::new(file);
	serde_json::to_writer(writer, sheet).expect("serialize spritesheet metadata");
}
