use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use winapi::um::wingdi::DeleteObject;

#[derive(Debug)]
pub struct GdiObject<T> {
    pointer: NonNull<T>,
}

impl<T> GdiObject<T> {
    /// create a `GdiObject` from an existing pointer, taking ownership of that object.
    ///
    /// `GdiObject` will take ownership of `pointer`.
    pub fn new(pointer: *mut T) -> GdiObject<T> {
        let pointer = NonNull::new(pointer).expect("cannot create `GdiObject` from null object.");

        GdiObject { pointer }
    }

    pub fn as_raw(&mut self) -> *mut T {
        self.pointer.as_ptr()
    }
}

impl<T> Deref for GdiObject<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T> DerefMut for GdiObject<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }
}

impl<T> Drop for GdiObject<T> {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.pointer.as_ptr() as *mut c_void);
        }
    }
}

/// calls `function`, which returns an instance of `T` on success or `null` on failure.
///
/// on failure, the failure reason is queried using `GetLastError`.
pub fn gdi_call<TFunction, T>(function: TFunction) -> Result<GdiObject<T>, std::io::Error>
where
    TFunction: FnOnce() -> *mut T,
{
    let pointer = function();

    match pointer.is_null() {
        true => Err(std::io::Error::last_os_error()),
        false => Ok(GdiObject::new(pointer)),
    }
}
