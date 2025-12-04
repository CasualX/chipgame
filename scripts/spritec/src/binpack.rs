
pub struct GridBinPacker {
	cell: i32,
	width_cells: i32,
	height_cells: i32,
	occupied: Vec<bool>,
}

impl GridBinPacker {
	pub fn new(total_width: i32, total_height: i32, cell: i32) -> Self {
		assert!(total_width % cell == 0 && total_height % cell == 0, "sheet aligns to grid");
		let width_cells = total_width / cell;
		let height_cells = total_height / cell;
		let occupied = vec![false; (width_cells * height_cells) as usize];
		Self {
			cell,
			width_cells,
			height_cells,
			occupied,
		}
	}

	pub fn insert(&mut self, width: i32, height: i32) -> Option<(i32, i32)> {
		let w_cells = ceil_div(width, self.cell);
		let h_cells = ceil_div(height, self.cell);
		if w_cells > self.width_cells || h_cells > self.height_cells {
			return None;
		}
		for y in 0..=self.height_cells - h_cells {
			for x in 0..=self.width_cells - w_cells {
				if self.is_free(x, y, w_cells, h_cells) {
					self.mark(x, y, w_cells, h_cells);
					return Some((x * self.cell, y * self.cell));
				}
			}
		}
		None
	}

	fn is_free(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
		for yy in y..y + h {
			for xx in x..x + w {
				let idx = (yy * self.width_cells + xx) as usize;
				if self.occupied[idx] {
					return false;
				}
			}
		}
		true
	}

	fn mark(&mut self, x: i32, y: i32, w: i32, h: i32) {
		for yy in y..y + h {
			for xx in x..x + w {
				let idx = (yy * self.width_cells + xx) as usize;
				self.occupied[idx] = true;
			}
		}
	}
}

fn ceil_div(a: i32, b: i32) -> i32 {
	assert!(a > 0 && b > 0);
	(a + b - 1) / b
}
