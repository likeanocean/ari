/// creates a path from several components. all components except for the first are treated as relative paths, even if
/// they begin with a leading slash.
///
/// # examples.
///
/// ```
/// # use ari::path;
/// # use std::path::PathBuf;
///
/// let path = path!["/var", "/bin", "/ari/hello.so"];
///
/// assert_eq!(path, PathBuf::from("/var/bin/ari/hello.so"));
/// ```
#[macro_export]
macro_rules! path {
    () => {
        ::std::path::PathBuf::new()
    };

    ($initial: expr) => {
        ::std::path::PathBuf::from($initial)
    };

    ($initial: expr, $($extra: expr),* $(,)*) => ({
        let mut path = ::std::path::PathBuf::from($initial);

        $(
            $crate::path::util::_ari_path_append(&mut path, $extra);
        )*

        path
    });
}
