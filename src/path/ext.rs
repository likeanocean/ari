use std::path::{Path, PathBuf};

pub trait PathBufExt {
    /// appends a `path` to self. always treats `path` as a relative path.
    ///
    /// # examples
    ///
    /// ```
    /// # use ari::path::PathBufExt;
    /// # use std::path::PathBuf;
    ///
    /// let mut path = PathBuf::from("/var/bin");
    /// path.append("/ari/hello.so");
    ///
    /// let expected = PathBuf::from("/var/bin/ari/hello.so");
    /// assert_eq!(path, expected);
    /// ```
    fn append(&mut self, path: impl AsRef<Path>);
}

impl PathBufExt for PathBuf {
    fn append(&mut self, path: impl AsRef<Path>) {
        crate::path::util::_ari_path_append(self, path);
    }
}
