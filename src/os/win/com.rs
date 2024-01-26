use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::ptr::NonNull;
use winapi::shared::guiddef::{IID, REFIID};
use winapi::shared::ntdef::HRESULT;
use winapi::um::unknwnbase::IUnknown;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Iid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl Iid {
    /// defines a constant iid.
    ///     let IUnknown = iid(0x00000000, 0x0000, 0x0000, [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46]);
    pub const fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Iid {
        Iid {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

impl Into<IID> for Iid {
    fn into(self) -> IID {
        unsafe { std::mem::transmute(self) }
    }
}

impl From<IID> for Iid {
    fn from(iid: IID) -> Iid {
        unsafe { std::mem::transmute(iid) }
    }
}

impl Deref for Iid {
    type Target = IID;

    fn deref(&self) -> &IID {
        unsafe { &*(self as *const Iid as *const IID) }
    }
}

impl DerefMut for Iid {
    fn deref_mut(&mut self) -> &mut IID {
        unsafe { &mut *(self as *mut Iid as *mut IID) }
    }
}

/// a com pointer.
pub struct ComPtr<T> {
    pointer: NonNull<T>,
}

impl<T> ComPtr<T> {
    /// create a `ComPtr` from an existing pointer.
    ///
    /// `ComPtr` will take ownership of `pointer` and invoke `IUnknown::Release` on destruction. `ComPtr::new` will not
    /// increment the pointer's reference count.
    ///
    /// `T` **must** inherit `IUnknown`.
    pub unsafe fn new(pointer: *mut T) -> ComPtr<T> {
        let pointer = NonNull::new(pointer).expect("cannot create `ComPtr` from null object.");

        ComPtr { pointer }
    }

    /// creates a `ComPtr` from an existing pointer, incrementing its reference count.
    ///
    /// `T` **must** inherit `IUnknown`.
    pub unsafe fn from(pointer: *mut T) -> ComPtr<T> {
        let pointer = NonNull::new(pointer).expect("cannot create `ComPtr` from null object.");

        reference_add(pointer.cast());

        ComPtr { pointer }
    }

    /// casts this pointer into `U`.
    pub fn cast<U>(&self) -> ComPtr<U>
    where
        T: Deref<Target = U>,
    {
        unsafe {
            reference_add(self.pointer.cast());

            ComPtr {
                pointer: self.pointer.cast(),
            }
        }
    }

    /// queries for the interface `U`.
    pub fn query<U: winapi::Interface>(&self) -> Result<ComPtr<U>, HRESULT> {
        self.query_iid::<U>(&U::uuidof())
    }

    /// queries for the interface named by specified `iid`.
    pub fn query_iid<U>(&self, iid: REFIID) -> Result<ComPtr<U>, HRESULT> {
        unsafe {
            let mut pointer = std::ptr::null_mut::<U>();
            let unknown = self.as_unknown();
            let hr =
                (*unknown).QueryInterface(iid, &mut pointer as *mut *mut _ as *mut *mut c_void);

            match hr >= 0 {
                true => Ok(ComPtr::new(pointer)),
                false => Err(hr),
            }
        }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }

    pub fn as_ptr(&self) -> *const T {
        self.pointer.as_ptr()
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }

    pub fn as_unknown(&self) -> *mut IUnknown {
        self.pointer.as_ptr() as *mut IUnknown
    }

    /// consumes this `ComPtr`, returning the wrapped pointer.
    ///
    /// after this function returns, the caller is responsible for the pointer instance previously managed by this
    /// `ComPtr`. callers should then invoke `IUnknown::Release` when done.
    pub fn into_raw(self) -> *mut T {
        let pointer = self.pointer.as_ptr();

        std::mem::forget(self);
        pointer
    }
}

impl<T> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.pointer.as_mut() }
    }
}

impl<T> Clone for ComPtr<T> {
    fn clone(&self) -> ComPtr<T> {
        unsafe {
            reference_add(self.pointer.cast());

            ComPtr {
                pointer: self.pointer,
            }
        }
    }
}

impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            reference_remove(self.pointer.cast());
        }
    }
}

impl<T> PartialEq<ComPtr<T>> for ComPtr<T> {
    fn eq(&self, other: &ComPtr<T>) -> bool {
        self.pointer == other.pointer
    }
}

impl<T> Debug for ComPtr<T> {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        formatter
            .debug_struct("ComPtr")
            .field("type", &std::any::type_name::<T>())
            .field("pointer", &self.pointer)
            .finish()
    }
}

pub unsafe fn reference_add(mut object: NonNull<IUnknown>) {
    object.as_mut().AddRef();
}

pub unsafe fn reference_remove(mut object: NonNull<IUnknown>) {
    object.as_mut().Release();
}
