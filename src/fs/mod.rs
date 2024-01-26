mod enumerate;
mod sys;

pub use self::enumerate::*;

use crate::io::ReadExt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// returns true if the file at `path` exists, and it is a file.
pub fn file_exists(path: impl AsRef<Path>) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => meta.is_file(),
        Err(_) => false,
    }
}

// returns true if the directory at `path` exists, and it is a directory.
pub fn directory_exists(path: impl AsRef<Path>) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => meta.is_dir(),
        Err(_) => false,
    }
}

/// opens a binary file, reads the contents of the file into a vec<u8>, and then closes the file.
pub fn read_all_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(path)?;

    file.read_as_bytes()
}

/// creates a new file, writes the specified byte slice to the file, and then closes the file. if the target file
/// already exists, it is overwritten.
pub fn write_all_bytes(path: impl AsRef<Path>, data: &[u8]) -> Result<(), std::io::Error> {
    File::create(path).and_then(|mut file| file.write_all(data))
}

/// opens a text file, reads the contents of the file into a string, and then closes the file.
pub fn read_all_text(path: impl AsRef<Path>) -> Result<String, std::io::Error> {
    let bytes = read_all_bytes(path)?;

    String::from_utf8(bytes).map_err(|_| std::io::ErrorKind::InvalidData.into())
}

/// opens a text file, reads the all lines of the file into a `vec<string>`, and then closes the file.
pub fn read_all_lines(path: impl AsRef<Path>) -> Result<Vec<String>, std::io::Error> {
    let mut lines = read_all_text(path)?
        .split('\n')
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();

    for line in &mut lines {
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(lines)
}

/// creates a new file, write the contents to the file, and then closes the file. if the target file already exists, it
/// is overwritten.
pub fn write_all_text(path: impl AsRef<Path>, data: String) -> Result<(), std::io::Error> {
    write_all_bytes(path, &data.into_bytes())
}

/// replaces `source` file with `destination` file, using `backup` as an intermediatary backup.
///
/// if `source`, `destination` or `backup` are on different volumes, this method will likely fail.
pub fn replace(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    backup: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    let source = source.as_ref();
    let destination = destination.as_ref();
    let backup = backup.as_ref();

    if crate::fs::file_exists(backup) {
        std::fs::remove_file(backup)?;
    }

    if crate::fs::file_exists(destination) {
        std::fs::rename(destination, backup)?;
    }

    match std::fs::rename(source, destination) {
        Ok(()) => {
            std::fs::remove_file(backup).ok();
            Ok(())
        }
        Err(e) => {
            std::fs::rename(backup, source).ok();
            Err(e)
        }
    }
}

// extension methods for `std::fs::File`
pub trait FileExt {
    // returns the number of bytes allocated for this file.
    fn allocation_size(&self) -> Result<u64, std::io::Error>;

    // allocates at least `size` bytes for this file. if the existing allocation is greater than `length`, then this
    // method has no effect.
    fn set_allocation_size(&self, length: u64) -> Result<(), std::io::Error>;
}

impl FileExt for File {
    fn allocation_size(&self) -> Result<u64, std::io::Error> {
        crate::fs::sys::get_allocation_size(self)
    }

    fn set_allocation_size(&self, length: u64) -> Result<(), std::io::Error> {
        crate::fs::sys::set_allocation_size(self, length)
    }
}

#[derive(Clone, Debug)]
pub struct VolumeInformation {
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub allocation_granularity: u64,
}

pub fn get_volume_information(path: impl AsRef<Path>) -> Result<VolumeInformation, std::io::Error> {
    crate::fs::sys::get_volume_information(path.as_ref())
}

pub fn volume_free_bytes(path: impl AsRef<Path>) -> Result<u64, std::io::Error> {
    get_volume_information(path).map(|x| x.free_bytes)
}

pub fn volume_available_bytes(path: impl AsRef<Path>) -> Result<u64, std::io::Error> {
    get_volume_information(path).map(|x| x.available_bytes)
}

pub fn volume_total_bytes(path: impl AsRef<Path>) -> Result<u64, std::io::Error> {
    get_volume_information(path).map(|x| x.total_bytes)
}

pub fn allocation_granularity(path: impl AsRef<Path>) -> Result<u64, std::io::Error> {
    get_volume_information(path).map(|x| x.allocation_granularity)
}
