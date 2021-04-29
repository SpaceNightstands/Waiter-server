use std::fmt::Debug;
use std::marker::PhantomPinned;
use std::pin::Pin;

pub struct SharedPointer<T> {
	pointer: *const T,
	_pin: PhantomPinned,
}

impl<T> SharedPointer<T> {
	pub(crate) unsafe fn new(pointer: &T) -> Pin<Self> {
		Pin::new_unchecked(SharedPointer {
			pointer: pointer as *const T,
			_pin: PhantomPinned,
		})
	}

	#[cfg(test)]
	unsafe fn get_pointer(&self) -> *const T {
		self.pointer
	}
}

impl<T> std::ops::Deref for SharedPointer<T> {
	type Target = T;
	fn deref<'ret>(&'ret self) -> &'ret Self::Target {
		unsafe { &*self.pointer }
	}
}

impl<T> std::clone::Clone for SharedPointer<T> {
	fn clone(&self) -> Self {
		Self {
			pointer: self.pointer,
			_pin: PhantomPinned,
		}
	}
}
impl<T> std::marker::Copy for SharedPointer<T> {}

unsafe impl<T> std::marker::Send for SharedPointer<T> {}

impl<T: Debug> Debug for SharedPointer<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("SharedPointer(")?;
		(&**self).fmt(f)?;
		f.write_str(")")
	}
}

#[test]
fn shared_pointer_single_threaded_test() {
	let owned = String::from("test");
	let shared = unsafe { SharedPointer::new(&owned) };
	assert_eq!(&*shared, "test");
}

#[test]
fn shared_pointer_multithreaded_test() {
	let owned = String::from("Test");
	let shared = unsafe { SharedPointer::new(&owned) };
	let is_equal = std::thread::spawn(move || &*shared == "Test")
		.join()
		.unwrap();
	assert!(is_equal);
}

#[test]
fn box_and_ref_test() {
	let (owned, reference) = get_box_and_ref();
	unsafe {
		assert_eq!(
			&*owned as *const String,
			Pin::into_inner_unchecked(reference).get_pointer()
		);
	};
}

#[cfg(test)]
fn get_box_and_ref() -> (Box<String>, Pin<SharedPointer<String>>) {
	let owned = Box::new(String::from("Test"));
	let shared = unsafe { SharedPointer::new(&*owned) };
	(owned, shared)
}
