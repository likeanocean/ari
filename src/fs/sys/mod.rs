#[cfg(windows)]
#[path = "win.rs"]
mod a;

#[cfg(unix)]
#[path = "unix.rs"]
mod a;


pub(crate) use self::a::*;
