// https://github.com/danburkert/fs2-rs/tree/9a340454a8292df025de368fc4b310bb736f382f

use std::fs::File;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use winapi::shared::minwindef::DWORD;
use winapi::um::fileapi::{GetDiskFreeSpaceW, GetVolumePathNameW, SetFileInformationByHandle};
use winapi::um::fileapi::{FILE_ALLOCATION_INFO, FILE_STANDARD_INFO};
use winapi::um::winbase::GetFileInformationByHandleEx;

use crate::fs::VolumeInformation;

/// returns the number of bytes allocated for this file.
pub(crate) fn get_allocation_size(file: &File) -> Result<u64, std::io::Error> {
    unsafe {
        let mut info = std::mem::zeroed::<FILE_STANDARD_INFO>();

        let handle = file.as_raw_handle();
        let class = winapi::um::minwinbase::FileStandardInfo;
        let data = &mut info as *mut _ as *mut _;
        let size = std::mem::size_of::<FILE_STANDARD_INFO>() as DWORD;

        match GetFileInformationByHandleEx(handle, class, data, size) {
            0 => Err(std::io::Error::last_os_error()),
            _ => Ok(*info.AllocationSize.QuadPart() as u64),
        }
    }
}

/// allocates at least `size` bytes for this file. if the existing allocation is greater than `length`, then this method
/// has no effect.
pub(crate) fn set_allocation_size(file: &File, size: u64) -> Result<(), std::io::Error> {
    if get_allocation_size(file)? < size {
        unsafe {
            let mut info = std::mem::zeroed::<FILE_ALLOCATION_INFO>();

            *info.AllocationSize.QuadPart_mut() = size as i64;

            let handle = file.as_raw_handle();
            let class = winapi::um::minwinbase::FileAllocationInfo;
            let data = &mut info as *mut _ as *mut _;
            let size = std::mem::size_of::<FILE_ALLOCATION_INFO>() as DWORD;

            match SetFileInformationByHandle(handle, class, data, size) {
                0 => Err(std::io::Error::last_os_error()),
                _ => Ok(()),
            }
        }
    } else {
        Ok(())
    }
}

pub(crate) fn get_volume_information(path: &Path) -> Result<VolumeInformation, std::io::Error> {
    let volume: &mut [u16] = &mut [0; 265];

    unsafe {
        put_volume_into(volume, path)?;

        let mut sectors_per_cluster = 0;
        let mut bytes_per_sector = 0;
        let mut free_clusters = 0;
        let mut total_clusters = 0;

        if GetDiskFreeSpaceW(
            volume.as_ptr(),
            &mut sectors_per_cluster,
            &mut bytes_per_sector,
            &mut free_clusters,
            &mut total_clusters,
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }

        let cluster_size = sectors_per_cluster as u64 * bytes_per_sector as u64;
        let available_bytes = cluster_size * free_clusters as u64;
        let total_bytes = cluster_size * total_clusters as u64;

        return Ok(VolumeInformation {
            free_bytes: available_bytes,
            available_bytes: available_bytes,
            total_bytes: total_bytes,
            allocation_granularity: cluster_size,
        });
    }

    fn put_volume_into(into: &mut [u16], path: &Path) -> Result<(), std::io::Error> {
        let utf16 = crate::os::win::to_utf16_null(path);

        unsafe {
            match GetVolumePathNameW(utf16.as_ptr(), into.as_mut_ptr(), into.len() as DWORD) {
                0 => Err(std::io::Error::last_os_error()),
                _ => Ok(()),
            }
        }
    }
}
