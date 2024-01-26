use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::{DirEntry, FileType, Metadata, ReadDir};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SearchOption {
    TopOnly,
    Recursive,
}

pub fn entries(
    path: impl AsRef<Path>,
    option: SearchOption,
) -> Result<impl Iterator<Item = Result<FsEntry, std::io::Error>> + Debug, std::io::Error> {
    Enumerator::new(path.as_ref(), option)
}

pub fn directories(
    path: impl AsRef<Path>,
    option: SearchOption,
) -> Result<impl Iterator<Item = Result<FsEntry, std::io::Error>> + Debug, std::io::Error> {
    filtered_entries(path, option, |x| x.is_dir())
}

pub fn files(
    path: impl AsRef<Path>,
    option: SearchOption,
) -> Result<impl Iterator<Item = Result<FsEntry, std::io::Error>> + Debug, std::io::Error> {
    filtered_entries(path, option, |x| x.is_file())
}

fn filtered_entries(
    path: impl AsRef<Path>,
    option: SearchOption,
    predicate: impl Fn(FileType) -> bool,
) -> Result<impl Iterator<Item = Result<FsEntry, std::io::Error>> + Debug, std::io::Error> {
    macro_rules! bubble {
        ($expression: expr) => {{
            match $expression {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            }
        }};
    }

    entries(path, option).map(move |sequence| {
        sequence.filter_map(move |r| {
            let entry = bubble![r];
            let ty = entry.ty();

            if predicate(ty) {
                Some(Ok(entry))
            } else {
                None
            }
        })
    })
}

#[derive(Debug)]
struct Enumerator {
    stack: Vec<Directory>,
    recursive: bool,
}

impl Enumerator {
    fn new(directory: &Path, option: SearchOption) -> Result<Enumerator, std::io::Error> {
        let source = std::fs::read_dir(directory)?;
        let stack = vec![Directory { source: Ok(source) }];
        let recursive = match option {
            SearchOption::Recursive => true,
            SearchOption::TopOnly => false,
        };

        Ok(Enumerator { stack, recursive })
    }
}

impl Iterator for Enumerator {
    type Item = Result<FsEntry, std::io::Error>;

    fn next(&mut self) -> Option<Result<FsEntry, std::io::Error>> {
        while !self.stack.is_empty() {
            match self.stack.last_mut().expect("!").next() {
                None => {
                    self.stack.pop();
                }

                Some(Err(error)) => {
                    return Some(Err(error));
                }

                Some(Ok(entry)) => {
                    if self.recursive && entry.ty().is_dir() {
                        let path = entry.path();
                        let directory = Directory::new(path);

                        self.stack.push(directory);
                    }

                    return Some(Ok(entry));
                }
            }
        }

        None
    }
}

// an iterator that yields a sequence of fs-entries (instead of `std::fs::DirEntry`).
#[derive(Debug)]
struct Directory {
    source: Result<ReadDir, Option<std::io::Error>>,
}

impl Directory {
    fn new(path: PathBuf) -> Directory {
        Directory {
            source: std::fs::read_dir(path).map_err(Some),
        }
    }
}

impl Iterator for Directory {
    type Item = Result<FsEntry, std::io::Error>;

    fn next(&mut self) -> Option<Result<FsEntry, std::io::Error>> {
        match self.source {
            Err(ref mut e) => e.take().map(Err),
            Ok(ref mut i) => i.next().map(|x| x.and_then(FsEntry::new)),
        }
    }
}

// an fs-entry.
#[derive(Debug)]
pub struct FsEntry {
    ty: FileType,
    entry: DirEntry,
}

impl FsEntry {
    fn new(entry: DirEntry) -> Result<FsEntry, std::io::Error> {
        let ty = entry.file_type()?;

        Ok(FsEntry { entry, ty })
    }

    pub fn path(&self) -> PathBuf {
        self.entry.path()
    }

    pub fn name(&self) -> OsString {
        self.entry.file_name()
    }

    pub fn ty(&self) -> FileType {
        self.ty
    }

    pub fn metadata(&self) -> Result<Metadata, std::io::Error> {
        self.entry.metadata()
    }
}
