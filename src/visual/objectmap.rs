use super::*;

#[derive(Default)]
pub struct ObjectMap {
	pub map: HashMap<ObjectHandle, Object>,
	pub lookup: HashMap<core::EntityHandle, ObjectHandle>,
	pub next: ObjectHandle,
}
impl ObjectMap {
	pub fn alloc(&mut self) -> ObjectHandle {
		self.next.0 += 1;
		return self.next;
	}
	pub fn create(&mut self, obj: Object) -> ObjectHandle {
		self.next.0 += 1;
		let handle = self.next;
		self.map.insert(handle, Object { handle, ..obj });
		return handle;
	}
	pub fn insert(&mut self, obj: Object) {
		assert_ne!(obj.handle.0, 0, "Object handle is zero, use alloc() or create() to allocate a new handle.");
		self.map.insert(obj.handle, obj);
	}
	pub fn get(&self, handle: ObjectHandle) -> Option<&Object> {
		self.map.get(&handle)
	}
	pub fn get_mut(&mut self, handle: ObjectHandle) -> Option<&mut Object> {
		self.map.get_mut(&handle)
	}
	pub fn remove(&mut self, handle: ObjectHandle) -> Option<Object> {
		self.map.remove(&handle)
	}
	pub fn with<F: FnMut(&mut Object)>(&mut self, handle: ObjectHandle, mut f: F) -> bool {
		if let Some(mut ent) = self.map.remove(&handle) {
			f(&mut ent);
			self.map.insert(ent.handle, ent);
			true
		}
		else {
			false
		}
	}
	pub fn find_handle(&self, kind: core::EntityKind) -> Option<ObjectHandle> {
		for ent in self.map.values() {
			if ent.entity_kind == kind {
				return Some(ent.handle);
			}
		}
		None
	}
}
