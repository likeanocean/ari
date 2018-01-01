use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use winapi::shared::minwindef::{FARPROC, HMODULE};
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress, LoadLibraryW};


pub fn module_handle(name: &str) -> Option<HMODULE> {
    unsafe {
        let name = crate::os::win::to_utf16_null(name);
        let handle = GetModuleHandleW(name.as_ptr());

        match handle.is_null() {
            true => None,
            false => Some(handle),
        }
    }
}


#[derive(Debug)]
pub struct Library {
    handle: HMODULE,
}

impl Library {
    pub fn open(name: &str) -> Result<Library, std::io::Error> {
        unsafe {
            let name = crate::os::win::to_utf16_null(name);
            let handle = LoadLibraryW(name.as_ptr());

            match !handle.is_null() {
                true => Ok(Library { handle }),
                false => Err(std::io::Error::last_os_error()),
            }
        }
    }

    pub unsafe fn find<T>(&self, name: &[u8]) -> Result<Symbol<T>, std::io::Error> {
        let pointer = GetProcAddress(self.handle, name.as_ptr() as *const i8);

        match !pointer.is_null() {
            true => Ok(Symbol {
                pointer,
                phantom: PhantomData,
            }),
            false => Err(std::io::Error::last_os_error()),
        }
    }

    pub fn as_raw(&self) -> HMODULE {
        self.handle
    }
}


#[derive(Clone)]
pub struct Symbol<T> {
    pointer: FARPROC,
    phantom: PhantomData<T>,
}

impl<T> Deref for Symbol<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // safe: `symbol` can only be constructed from an unsafe context, and `self.pointer` is guaranteed to non-null
        unsafe { std::mem::transmute(&self.pointer) }
    }
}

impl<T> Debug for Symbol<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct("Symbol")
            .field("type", &std::any::type_name::<T>())
            .field("address", &format_args!("{:x}", self.pointer as usize))
            .finish()
    }
}

unsafe impl<T: Send> Send for Symbol<T> {
}

unsafe impl<T: Sync> Sync for Symbol<T> {
}
