use std::ffi::OsStr;
use std::path::{Path, PathBuf};


pub fn _ari_path_append(destination: &mut PathBuf, path: impl AsRef<Path>) {
    let path = path.as_ref();
    let os = path.as_os_str();
    let bytes = os_str_as_u8_slice(os);

    if bytes.is_empty() {
        destination.push(os);
    } else {
        for bit in bytes.split(|x| *x == b'/' || *x == b'\\') {
            destination.push(unsafe { u8_slice_as_os_str(bit) });
        }
    }
}


// why this is ok: https://github.com/rust-lang/rust/blob/b16c7a235fa0f57fed6b7ec13ffd3cff1bcdd9ad/src/libstd/path.rs#L88
fn os_str_as_u8_slice(os: &OsStr) -> &[u8] {
    unsafe { &*(os as *const OsStr as *const [u8]) }
}

unsafe fn u8_slice_as_os_str(slice: &[u8]) -> &OsStr {
    &*(slice as *const [u8] as *const OsStr)
}
