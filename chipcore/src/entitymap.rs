use super::*;

/// Indirect entity reference.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct EntityHandle(u32);

impl EntityHandle {
	pub const INVALID: EntityHandle = EntityHandle(0);

	#[inline]
	fn index(self) -> Option<usize> {
		if self.0 == 0 {
			return None;
		}
		return Some(self.0 as usize - 1);
	}
}

#[derive(Clone)]
enum Slot {
	Free { next: usize },
	Occupied { ent: Entity },
	Taken,
}

impl Slot {
	fn as_ent(&self) -> Option<&Entity> {
		match self {
			Slot::Occupied { ent } => Some(ent),
			_ => None,
		}
	}
	fn as_mut_ent(&mut self) -> Option<&mut Entity> {
		match self {
			Slot::Occupied { ent } => Some(ent),
			_ => None,
		}
	}
}

/// Entity map.
///
/// Has ownership of all entities in the game.  
/// Manages relationship between entity handles and entities.
#[derive(Clone, Default)]
pub struct EntityMap {
	slots: Vec<Slot>,
	next: usize,
}

impl EntityMap {
	/// Returns true if the ehandle is valid.
	pub fn is_valid(&self, ehandle: EntityHandle) -> bool {
		let Some(index) = ehandle.index() else { return false };
		let Some(slot) = self.slots.get(index) else { return false };
		return matches!(slot, Slot::Occupied { ent } if ent.handle == ehandle);
	}
	/// Allocates a new ehandle.
	///
	/// The entity is a placeholder and must be initialized by [`EntityMap::put`].
	pub fn alloc(&mut self) -> EntityHandle {
		let index = self.next;
		if index >= self.slots.len() {
			self.slots.push(Slot::Free { next: index + 1 });
		}
		let Some(slot) = self.slots.get_mut(index) else {
			broken_entity_map();
		};
		let Slot::Free { next } = slot else {
			broken_entity_map();
		};
		self.next = *next;
		*slot = Slot::Taken;
		return EntityHandle((index + 1) as u32);
	}
	/// Removes an entity by ehandle.
	///
	/// The ehandle is invalidated and can be reused by [`EntityMap::alloc`].
	///
	/// Danger! Use [`entity_remove`] instead!
	pub(super) fn remove(&mut self, ehandle: EntityHandle) -> Option<Entity> {
		let index = ehandle.index()?;
		let slot = self.slots.get_mut(index)?;
		// Only operate on truly occupied slots
		let Slot::Occupied { ent } = slot else {
			return None;
		};
		if ent.handle != ehandle {
			entity_handle_mismatch(ehandle);
		};
		// Move the entity out of the slot, and mark the slot as free
		let Slot::Occupied { mut ent } = mem::replace(slot, Slot::Free { next: self.next }) else {
			return None;
		};
		self.next = index;
		ent.handle = EntityHandle::INVALID;
		return Some(ent);
	}
	/// Gets an entity by ehandle.
	pub fn get(&self, ehandle: EntityHandle) -> Option<&Entity> {
		let index = ehandle.index()?;
		self.slots.get(index).and_then(Slot::as_ent)
	}
	/// Gets a mutable entity by ehandle.
	pub fn get_mut(&mut self, ehandle: EntityHandle) -> Option<&mut Entity> {
		let index = ehandle.index()?;
		self.slots.get_mut(index).and_then(Slot::as_mut_ent)
	}
	/// Takes an entity out by ehandle.
	///
	/// This allows the entity to be updated with a mutable [`GameState`].
	pub fn take(&mut self, ehandle: EntityHandle) -> Option<Entity> {
		let index = ehandle.index()?;
		let slot = self.slots.get_mut(index)?;
		// Only operate on truly occupied slots
		let Slot::Occupied { ent } = slot else {
			return None;
		};
		if ent.handle != ehandle {
			entity_handle_mismatch(ehandle);
		};
		// Move the entity out of the slot without cloning, and mark the slot as taken
		let Slot::Occupied { ent } = mem::replace(slot, Slot::Taken) else {
			return None;
		};
		return Some(ent);
	}
	/// Puts an entity back.
	///
	/// Can only be used with entities taken out by [`EntityMap::take`] and [`EntityMap::alloc`].
	pub fn put(&mut self, ent: Entity) {
		let Some(slot) = ent.handle.index().and_then(|index| self.slots.get_mut(index)) else {
			invalid_entity_handle(ent.handle);
		};
		let Slot::Taken = slot else {
			invalid_entity_handle(ent.handle);
		};
		*slot = Slot::Occupied { ent };
	}
	/// Iterates over all entity handles.
	pub fn handles(&self) -> impl Iterator<Item = EntityHandle> + Clone {
		(0..self.slots.len()).map(|index| EntityHandle((index + 1) as u32))
	}
	/// Iterates over all entities.
	pub fn iter(&self) -> impl Iterator<Item = &Entity> {
		self.slots.iter().filter_map(Slot::as_ent)
	}
	/// Iterates over all entities mutably.
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
		self.slots.iter_mut().filter_map(Slot::as_mut_ent)
	}
	/// Removes all entities.
	pub fn clear(&mut self) {
		self.slots.clear();
		self.next = 0;
	}
}

#[cold]
#[track_caller]
fn invalid_entity_handle(ehandle: EntityHandle) -> ! {
	panic!("Invalid entity handle: {:?}", ehandle);
}

#[cold]
#[track_caller]
fn entity_handle_mismatch(ehandle: EntityHandle) -> ! {
	panic!("Entity handle mismatch: {:?}", ehandle);
}

#[cold]
#[track_caller]
fn broken_entity_map() -> ! {
	panic!("Broken entity map detected");
}
