use super::*;

pub fn left_click(s: &mut EditorState, pressed: bool) {
	let cursor_pos = s.cursor_pos;
	if pressed {
		if cursor_pos.x < 0 || cursor_pos.y < 0 {
			s.sample();
		}
		else {
			// Sample from the existing entities
			let ehandle = s.game.gs.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
			if let Some(ehandle) = ehandle {
				s.tool = Tool::Entity;
				s.selected_ent = ehandle;
				s.selected_args = None;
				if let Some(ent) = s.game.gs.ents.get(ehandle) {
					s.selected_args = Some(ent.to_entity_args());
					println!("Selected: {:?} at {}", ent.kind, ent.pos);
				}
			}
			// Otherwise create a new entity
			else {
				if let Some(args) = s.selected_args {
					s.selected_ent = s.game.gs.entity_create(&core::EntityArgs { kind: args.kind, pos: cursor_pos, face_dir: args.face_dir });
				}
				s.game.sync(None);
			}
		}
	}
	else {
		// If we have a selected entity and the cursor has moved, move the entity
		if let Some(ent) = s.game.gs.ents.get(s.selected_ent) {
			if ent.pos != cursor_pos {
				if let Some(args) = s.game.gs.entity_remove(s.selected_ent) {
					s.game.gs.entity_create(&core::EntityArgs { kind: args.kind, pos: cursor_pos, face_dir: args.face_dir });
					s.game.sync(None);
				}
			}
		}
	}
}

pub fn think(s: &mut EditorState) {
	// if s.input.left_click {
	// 	if let Some(args) = s.selected_args {
	// 		s.selected_ent = s.game.gs.entity_create(&core::EntityArgs { kind: args.kind, pos: s.cursor_pos, face_dir: args.face_dir });
	// 		s.game.sync(None);
	// 	}
	// }
}

pub fn right_click(s: &mut EditorState, pressed: bool) {
	let cursor_pos = s.cursor_pos;
	if pressed {
		// Sample from the existing entities
		let ehandle = s.game.gs.ents.iter().find_map(|ent| if ent.pos == cursor_pos { Some(ent.handle) } else { None });
		if let Some(ehandle) = ehandle {
			// First select the entity
			s.tool = Tool::Entity;
			s.selected_ent = ehandle;
			s.selected_args = None;
			if let Some(ent) = s.game.gs.ents.get(ehandle) {
				s.selected_args = Some(ent.to_entity_args());
				// Then rotate the entity
				let kind = ent.kind;
				let pos = ent.pos;
				if let Some(args) = s.game.gs.entity_remove(s.selected_ent) {
					let new_args = core::EntityArgs { kind, pos, face_dir: next_face_dir(args.face_dir) };
					s.selected_args = Some(new_args);
					s.selected_ent = s.game.gs.entity_create(&new_args);
					println!("Rotated: {:?} at {}", kind, pos);
					s.game.sync(None);
				}
			}
		}
	}
}

pub fn delete(s: &mut EditorState, pressed: bool) {
	if pressed {
		if let Some(ent) = s.game.gs.ents.get(s.selected_ent) {
			let kind = ent.kind;
			let pos = ent.pos;
			s.game.gs.entity_remove(s.selected_ent);
			s.game.sync(None);
			println!("Deleted: {:?} at {}", kind, pos);
		}
		s.selected_ent = core::EntityHandle::INVALID;
	}
}
