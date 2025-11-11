use super::*;

/// Object handle type.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct ObjectHandle(u32);

/// Collection of objects.
#[derive(Default)]
pub struct ObjectMap {
	map: HashMap<ObjectHandle, Object>,
	next: u32,
}
impl ObjectMap {
	#[inline]
	pub fn alloc(&mut self) -> ObjectHandle {
		self.next += 1;
		return ObjectHandle(self.next);
	}
	#[inline]
	pub fn insert(&mut self, handle: ObjectHandle, obj: Object) {
		assert_ne!(handle.0, 0, "Object handle is zero, use alloc() or create() to allocate a new handle.");
		self.map.insert(handle, obj);
	}
	#[inline]
	pub fn get(&self, handle: ObjectHandle) -> Option<&Object> {
		self.map.get(&handle)
	}
	#[inline]
	pub fn get_mut(&mut self, handle: ObjectHandle) -> Option<&mut Object> {
		self.map.get_mut(&handle)
	}
	#[inline]
	pub fn remove(&mut self, handle: ObjectHandle) -> Option<Object> {
		self.map.remove(&handle)
	}
	#[inline]
	pub fn values(&self) -> impl Iterator<Item = &Object> {
		self.map.values().filter(|obj| obj.visible)
	}
	#[inline]
	pub fn retain<F: FnMut(&ObjectHandle, &mut Object) -> bool>(&mut self, f: F) {
		self.map.retain(f);
	}
	#[inline]
	pub fn clear(&mut self) {
		self.map.clear();
		self.next = 0;
	}
}
