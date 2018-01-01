mod handle;
mod internal;
mod library;

pub mod com;
pub mod gdi;
pub mod hr;
pub mod process;


pub use self::com::{ComPtr, Iid};
pub use self::gdi::GdiObject;
pub use self::handle::{GenericHandle, GenericHandleDtor, WindowsHandle};
pub use self::library::{module_handle, Library, Symbol};


use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use winapi::um::dwmapi::DwmIsCompositionEnabled;
use winapi::um::sysinfoapi::GetVersionExW;
use winapi::um::winnt::OSVERSIONINFOW;


// :: initialization.

crate fn initialize() {
    crate::os::win::internal::wer::disable_windows_error_reporting();
    crate::os::win::internal::vt::enable_vt_mode();
    crate::os::win::internal::dpi::activate_dpi_awareness();
}

// :: string-related methods.

pub fn to_utf16(string: impl AsRef<OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().collect()
}

pub fn to_utf16_null(string: impl AsRef<OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(Some(0)).collect()
}


pub fn from_utf16(data: &[u16]) -> OsString {
    OsString::from_wide(data)
}

pub fn from_utf16_null(data: &[u16]) -> OsString {
    let length = data.iter().position(|x| *x == 0).unwrap_or(data.len());

    OsString::from_wide(&data[..length])
}


pub unsafe fn from_utf16_ptr(data: *const u16, length: usize) -> OsString {
    assert![!data.is_null()];

    let slice = std::slice::from_raw_parts(data, length);
    OsString::from_wide(slice)
}

pub unsafe fn from_utf16_ptr_null(data: *const u16) -> OsString {
    assert![!data.is_null()];

    let length = (0..std::isize::MAX)
        .position(|i| *data.offset(i) == 0)
        .expect("data must be null terminated");

    let slice = std::slice::from_raw_parts(data, length);

    OsString::from_wide(slice)
}


// :: os versions and features.

pub fn aero_enabled() -> Result<bool, std::io::Error> {
    unsafe { crate::os::win::hr::call(|x| DwmIsCompositionEnabled(x)).map(|x| x != 0) }
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OsVersion {
    pub major: usize,
    pub minor: usize,
    pub build: usize,
}

impl OsVersion {
    pub const fn new(major: usize, minor: usize, build: usize) -> OsVersion {
        OsVersion { major, minor, build }
    }

    pub const fn windows_7() -> OsVersion {
        OsVersion::new(6, 1, 0)
    }

    pub const fn windows_8() -> OsVersion {
        OsVersion::new(6, 2, 0)
    }

    pub const fn windows_8_1() -> OsVersion {
        OsVersion::new(6, 3, 0)
    }

    pub const fn windows_10() -> OsVersion {
        OsVersion::new(10, 0, 0)
    }
}

pub fn os_version() -> Result<OsVersion, std::io::Error> {
    let x = unsafe {
        crate::os::win::hr::call(|x: *mut OSVERSIONINFOW| {
            (*x).dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

            GetVersionExW(x) != 0
        })
    }?;

    Ok(OsVersion {
        major: x.dwMajorVersion as usize,
        minor: x.dwMinorVersion as usize,
        build: x.dwBuildNumber as usize,
    })
}
