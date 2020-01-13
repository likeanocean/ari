// https://github.com/danburkert/fs2-rs/tree/9a340454a8292df025de368fc4b310bb736f382f

use std::ffi::CString;
use std::fs::File;
use std::os::unix::{ffi::OsStrExt, fs::MetadataExt, io::AsRawFd};
use std::path::Path;

use crate::fs::VolumeInformation;


/// returns the number of bytes allocated for this file.
crate fn get_allocation_size(file: &File) -> Result<u64, std::io::Error> {
    file.metadata().map(|x| x.blocks() as u64 * 512)
}


// #[cfg(any(target_os = "linux", target_os = "nacl"))]
// crate fn set_allocation_size(file: &File, size: u64) -> Result<(), std::io::Error> {
//     match unsafe { libc::fallocate(file.as_raw_fd(), libc::FALLOC_FL_KEEP_SIZE, 0, size as libc::off_t) } {
//         0 => Ok(()),
//         _ => Err(std::io::Error::last_os_error()),
//     }
// }

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "android",
    target_os = "emscripten",
    target_os = "nacl"
))]
crate fn set_allocation_size(file: &File, size: u64) -> Result<(), std::io::Error> {
    match unsafe { libc::posix_fallocate(file.as_raw_fd(), 0, size as libc::off_t) } == 0 {
        true => Ok(()),
        false => Err(std::io::Error::last_os_error()),
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
crate fn set_allocation_size(file: &File, size: u64) -> Result<(), std::io::Error> {
    let metadata = file.metadata()?;

    if size > metadata.blocks() as u64 * 512 {
        let mut fstore = libc::fstore_t {
            fst_flags:      libc::F_ALLOCATECONTIG,
            fst_posmode:    libc::F_PEOFPOSMODE,
            fst_offset:     0,
            fst_length:     size as libc::off_t,
            fst_bytesalloc: 0,
        };

        if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_PREALLOCATE, &fstore) } == -1 {
            // contiguous allocation failed, attempt to allocate non-contiguously.
            fstore.fst_flags = libc::F_ALLOCATEALL;

            if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_PREALLOCATE, &fstore) } == -1 {
                return Err(std::io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "solaris",
    target_os = "haiku"
))]
crate fn set_allocation_size(file: &File, size: u64) -> Result<(), std::io::Error> {
    // no allocation api is available on these operating systems.
    Ok(())
}


crate fn get_volume_information(path: &Path) -> Result<VolumeInformation, std::io::Error> {
    let data = path.as_os_str().as_bytes();
    let string = match CString::new(data) {
        Ok(x) => x,
        Err(_) => return Err(std::io::ErrorKind::InvalidInput.into()),
    };

    unsafe {
        let mut stat = std::mem::zeroed::<libc::statvfs>();

        // cast is necessary for platforms where libc::char != u8.
        if libc::statvfs(string.as_ptr() as *const _, &mut stat) == 0 {
            let information = VolumeInformation {
                free_bytes:             stat.f_frsize as u64 * stat.f_bfree as u64,
                available_bytes:        stat.f_frsize as u64 * stat.f_bavail as u64,
                total_bytes:            stat.f_frsize as u64 * stat.f_blocks as u64,
                allocation_granularity: stat.f_frsize as u64,
            };

            Ok(information)
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}
