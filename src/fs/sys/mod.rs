#[cfg(windows)]
#[path = "win.rs"]
mod a;

#[cfg(unix)]
#[path = "unix.rs"]
mod a;


crate use self::a::*;
