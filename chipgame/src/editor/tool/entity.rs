use super::*;

#[derive(Clone, Default)]
pub struct EntityToolState {
	pub selected_ent: EntityHandle,
	pub selected_args: Option<EntityArgs>,
}

impl fmt::Display for EntityToolState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if let Some(args) = &self.selected_args {
			write!(f, "{:?}", args.kind)
		}
		else {
			f.write_str("None")
		}
	}
}

impl EntityToolState {
	pub fn left_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		let cursor_pos = s.cursor_pos;
		if pressed {
			// Sample from the existing entities
			let ehandle = s.fx.game.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
			if let Some(ehandle) = ehandle {
				self.selected_ent = ehandle;
				self.selected_args = None;
				if let Some(ent) = s.fx.game.ents.get(ehandle) {
					self.selected_args = Some(ent.to_entity_args());
					println!("Selected: {:?} at {}", ent.kind, ent.pos);
				}
			}
			// Otherwise create a new entity
			else {
				if let Some(args) = self.selected_args {
					self.selected_ent = s.fx.game.entity_create(&EntityArgs { kind: args.kind, pos: cursor_pos, face_dir: args.face_dir });
				}
				s.fx.sync();
			}
		}
		else {
			// If we have a selected entity and the cursor has moved, move the entity
			if let Some(ent) = s.fx.game.ents.get(self.selected_ent) {
				if ent.pos != cursor_pos {
					if let Some(args) = s.fx.game.entity_remove(self.selected_ent) {
						s.fx.game.entity_create(&EntityArgs { kind: args.kind, pos: cursor_pos, face_dir: args.face_dir });
						s.fx.sync();
					}
				}
			}
		}
	}

	pub fn think(&mut self, _s: &mut EditorEditState) {
		// if s.input.left_click {
		// 	if let Some(args) = s.selected_args {
		// 		s.selected_ent = s.fx.game.entity_create(&EntityArgs { kind: args.kind, pos: s.cursor_pos, face_dir: args.face_dir });
		// 		s.fx.sync();
		// 	}
		// }
	}

	pub fn right_click(&mut self, s: &mut EditorEditState, pressed: bool) {
		let cursor_pos = s.cursor_pos;
		if pressed {
			// Sample from the existing entities
			let ehandle = s.fx.game.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
			if let Some(ehandle) = ehandle {
				// First select the entity
				self.selected_ent = ehandle;
				self.selected_args = None;
				if let Some(ent) = s.fx.game.ents.get(ehandle) {
					self.selected_args = Some(ent.to_entity_args());
					// Then rotate the entity
					let kind = ent.kind;
					let pos = ent.pos;
					if let Some(args) = s.fx.game.entity_remove(self.selected_ent) {
						let new_args = EntityArgs { kind, pos, face_dir: next_face_dir(args.face_dir) };
						self.selected_args = Some(new_args);
						self.selected_ent = s.fx.game.entity_create(&new_args);
						println!("Rotated: {:?} at {}", kind, pos);
						s.fx.sync();
					}
				}
			}
		}
	}

	pub fn delete(&mut self, s: &mut EditorEditState, pressed: bool) {
		if pressed {
			if self.selected_ent == EntityHandle::INVALID {
				let cursor_pos = s.cursor_pos;
				self.selected_ent = s.fx.game.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None }).unwrap_or(EntityHandle::INVALID);
			}
			if let Some(ent) = s.fx.game.ents.get(self.selected_ent) {
				let kind = ent.kind;
				let pos = ent.pos;
				s.fx.game.entity_remove(self.selected_ent);
				s.fx.sync();
				println!("Deleted: {:?} at {}", kind, pos);
			}
			self.selected_ent = EntityHandle::INVALID;
		}
	}
}

fn next_face_dir(face_dir: Option<Compass>) -> Option<Compass> {
	match face_dir {
		Some(Compass::Up) => Some(Compass::Right),
		Some(Compass::Right) => Some(Compass::Down),
		Some(Compass::Down) => Some(Compass::Left),
		Some(Compass::Left) => None,
		None => Some(Compass::Up),
	}
}
