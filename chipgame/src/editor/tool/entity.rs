use super::*;

pub fn left_click(s: &mut EditorEditState, pressed: bool) {
	let cursor_pos = s.cursor_pos;
	if pressed {
		if cursor_pos.x < 0 || cursor_pos.y < 0 {
			s.sample();
		}
		else {
			// Sample from the existing entities
			let ehandle = s.fx.game.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
			if let Some(ehandle) = ehandle {
				s.tool = Tool::Entity;
				s.selected_ent = ehandle;
				s.selected_args = None;
				if let Some(ent) = s.fx.game.ents.get(ehandle) {
					s.selected_args = Some(ent.to_entity_args());
					println!("Selected: {:?} at {}", ent.kind, ent.pos);
				}
			}
			// Otherwise create a new entity
			else {
				if let Some(args) = s.selected_args {
					s.selected_ent = s.fx.game.entity_create(&EntityArgs { kind: args.kind, pos: cursor_pos, face_dir: args.face_dir });
				}
				s.fx.sync();
			}
		}
	}
	else {
		// If we have a selected entity and the cursor has moved, move the entity
		if let Some(ent) = s.fx.game.ents.get(s.selected_ent) {
			if ent.pos != cursor_pos {
				if let Some(args) = s.fx.game.entity_remove(s.selected_ent) {
					s.fx.game.entity_create(&EntityArgs { kind: args.kind, pos: cursor_pos, face_dir: args.face_dir });
					s.fx.sync();
				}
			}
		}
	}
}

pub fn think(_s: &mut EditorEditState) {
	// if s.input.left_click {
	// 	if let Some(args) = s.selected_args {
	// 		s.selected_ent = s.fx.game.entity_create(&EntityArgs { kind: args.kind, pos: s.cursor_pos, face_dir: args.face_dir });
	// 		s.fx.sync();
	// 	}
	// }
}

pub fn right_click(s: &mut EditorEditState, pressed: bool) {
	let cursor_pos = s.cursor_pos;
	if pressed {
		// Sample from the existing entities
		let ehandle = s.fx.game.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
		if let Some(ehandle) = ehandle {
			// First select the entity
			s.tool = Tool::Entity;
			s.selected_ent = ehandle;
			s.selected_args = None;
			if let Some(ent) = s.fx.game.ents.get(ehandle) {
				s.selected_args = Some(ent.to_entity_args());
				// Then rotate the entity
				let kind = ent.kind;
				let pos = ent.pos;
				if let Some(args) = s.fx.game.entity_remove(s.selected_ent) {
					let new_args = EntityArgs { kind, pos, face_dir: next_face_dir(args.face_dir) };
					s.selected_args = Some(new_args);
					s.selected_ent = s.fx.game.entity_create(&new_args);
					println!("Rotated: {:?} at {}", kind, pos);
					s.fx.sync();
				}
			}
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

pub fn delete(s: &mut EditorEditState, pressed: bool) {
	if pressed {
		if s.selected_ent == EntityHandle::INVALID {
			let cursor_pos = s.cursor_pos;
			s.selected_ent = s.fx.game.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None }).unwrap_or(EntityHandle::INVALID);
		}
		if let Some(ent) = s.fx.game.ents.get(s.selected_ent) {
			let kind = ent.kind;
			let pos = ent.pos;
			s.fx.game.entity_remove(s.selected_ent);
			s.fx.sync();
			println!("Deleted: {:?} at {}", kind, pos);
		}
		s.selected_ent = EntityHandle::INVALID;
	}
}
