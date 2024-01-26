pub mod ext;
pub mod macros;
pub mod util;

pub use self::ext::*;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Component, Path, Prefix};

// returns the volume name for some path, intended for display.
pub fn volume_name(path: &Path) -> Option<String> {
    fn display(string: &OsStr) -> Cow<str> {
        string.to_string_lossy()
    }

    if let Some(component) = path.components().next() {
        let volume = match component {
            Component::Prefix(prefix) => match prefix.kind() {
                Prefix::Verbatim(name) => format!("{}", display(name)),
                Prefix::VerbatimUNC(server, share) => {
                    format!("{}\\{}", display(server), display(share))
                }
                Prefix::VerbatimDisk(disk) => format!("{}:\\", disk as char),
                Prefix::DeviceNS(namespace) => format!("{}", display(namespace)),
                Prefix::UNC(server, share) => format!("{}\\{}\\", display(server), display(share)),
                Prefix::Disk(disk) => format!("{}:\\", disk as char),
            },
            Component::RootDir => "/".to_owned(),
            Component::CurDir => ".".to_owned(),
            Component::ParentDir => "..".to_owned(),
            Component::Normal(x) => x.to_string_lossy().into(),
        };

        Some(volume)
    } else {
        None
    }
}
