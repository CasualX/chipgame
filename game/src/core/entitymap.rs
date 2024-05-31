use super::*;

/// Indirect entity reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct EntityHandle(u32);

impl EntityHandle {
	pub const INVALID: EntityHandle = EntityHandle(0xffffffff);
}

impl Default for EntityHandle {
	fn default() -> Self {
		EntityHandle::INVALID
	}
}

enum Slot {
	Free { next: EntityHandle },
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
pub struct EntityMap {
	slots: Vec<Slot>,
	next: EntityHandle,
}

impl Default for EntityMap {
	fn default() -> Self {
		EntityMap {
			slots: Vec::new(),
			next: EntityHandle(0),
		}
	}
}

impl EntityMap {
	/// Returns true if the ehandle is valid.
	pub fn is_valid(&self, ehandle: EntityHandle) -> bool {
		let Some(slot) = self.slots.get(ehandle.0 as usize) else { return false };
		return matches!(slot, Slot::Occupied { ent } if ent.handle == ehandle);
	}
	/// Allocates a new ehandle.
	///
	/// The entity is a placeholder and must be initialized by [`EntityMap::put`].
	pub fn alloc(&mut self) -> EntityHandle {
		let ehandle = self.next;
		if ehandle.0 as usize >= self.slots.len() {
			self.slots.push(Slot::Free { next: EntityHandle(ehandle.0 + 1) });
		}
		let Some(slot) = self.slots.get_mut(ehandle.0 as usize) else { return EntityHandle::INVALID };
		let Slot::Free { next } = *slot else { return EntityHandle::INVALID };
		self.next = next;
		*slot = Slot::Taken;
		return ehandle;
	}
	/// Removes an entity by ehandle.
	///
	/// The ehandle is invalidated and can be reused by [`EntityMap::alloc`].
	///
	/// Danger! Use [`entity_remove`] instead!
	pub(super) fn remove(&mut self, ehandle: EntityHandle) -> Option<Entity> {
		let slot = self.slots.get_mut(ehandle.0 as usize)?;
		let Slot::Occupied { ent } = slot else { return None };
		let mut ent = ent.clone();
		ent.handle = EntityHandle::INVALID;
		*slot = Slot::Free { next: self.next };
		self.next = ehandle;
		return Some(ent);
	}
	/// Gets an entity by ehandle.
	pub fn get(&self, ehandle: EntityHandle) -> Option<&Entity> {
		self.slots.get(ehandle.0 as usize).and_then(Slot::as_ent)
	}
	/// Gets a mutable entity by ehandle.
	pub fn get_mut(&mut self, ehandle: EntityHandle) -> Option<&mut Entity> {
		self.slots.get_mut(ehandle.0 as usize).and_then(Slot::as_mut_ent)
	}
	/// Takes an entity out by ehandle.
	///
	/// This allows the entity to be updated with a mutable [`GameState`].
	pub fn take(&mut self, ehandle: EntityHandle) -> Option<Entity> {
		let slot = self.slots.get_mut(ehandle.0 as usize)?;
		let Slot::Occupied { ent } = slot else { return None };
		if ent.handle != ehandle { return None };
		let ent = ent.clone();
		*slot = Slot::Taken;
		return Some(ent);
	}
	/// Puts an entity back.
	///
	/// Can only be used with entities taken out by [`EntityMap::take`] and [`EntityMap::alloc`].
	pub fn put(&mut self, ent: Entity) {
		let Some(slot) = self.slots.get_mut(ent.handle.0 as usize) else {
			panic!("Entity handle {:?} is not valid.", ent.handle)
		};
		let Slot::Taken = slot else {
			panic!("Entity handle {:?} is not taken.", ent.handle)
		};
		*slot = Slot::Occupied { ent };
	}
	/// Iterates over all entity handles.
	pub fn handles(&self) -> impl Iterator<Item = EntityHandle> + Clone {
		(0..self.slots.len() as u32).map(EntityHandle)
	}
	/// Iterates over all entities.
	pub fn iter(&self) -> impl Iterator<Item = &Entity> {
		self.slots.iter().filter_map(Slot::as_ent)
	}
	/// Iterates over all entities mutably.
	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
		self.slots.iter_mut().filter_map(Slot::as_mut_ent)
	}
}
