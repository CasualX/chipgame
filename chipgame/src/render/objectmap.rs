use super::*;

#[derive(Default)]
pub struct ObjectMap {
	pub(super) map: HashMap<ObjectHandle, Object>,
	pub(super) next: ObjectHandle,
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
	pub fn clear(&mut self) {
		self.map.clear();
		self.next = ObjectHandle(0);
	}
}
