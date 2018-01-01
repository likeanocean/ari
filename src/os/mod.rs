#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod win;


#[cfg(unix)]
pub fn initialize() {
    self::unix::initialize();
}

#[cfg(windows)]
pub fn initialize() {
    self::win::initialize();
}


#[cfg(windows)]
pub use crate::os::win::process::*;
