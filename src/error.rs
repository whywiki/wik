//! Shared wik error type

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::ObjectId;

/// `wik::Error` is shared wik error type
#[derive(Debug)]
pub enum Error {
    /// IO failures like missing file, permission denied, etc
    Io(io::Error),

    /// File system `path` failures
    Fs {
        /// What wik was doing (`"mkdir"`, `"write"`, ...)
        action: &'static str,

        /// The path of the failed operation
        path: PathBuf,

        /// What the operating system reported
        source: io::Error,
    },

    /// No repository was found in a directory or any of its parents
    NotARepository,

    /// File kind in the working folder cannot be stored yet
    Unsupported {
        /// File
        path: PathBuf,
        /// What kind of file it is
        what: &'static str,
    },

    /// Wrong object id name
    InvalidObjectName {
        /// Given name
        name: String,
    },

    /// Environment variable needed for a commit is not set or empty
    MissingVariable {
        /// variable for example `GIT_AUTHOR_NAME`
        name: &'static str,
    },

    /// Date is not in Git format `<seconds> <+hhmm>`
    InvalidDate {
        /// The date it was given
        date: String,
    },

    /// Object that should be in the repository isn't there
    MissingObject {
        /// id of the missing object
        id: ObjectId,
    },

    /// File is locked (its `.lock` file already exists)
    LockHeld {
        /// `.lock` file
        path: PathBuf,
    },
    /// Ref holds something that is not an id or `HEAD` points outside `refs/`
    BrokenRef {
        /// ref like `refs/heads/main` or `HEAD`
        name: String,
    },

    /// Repository keeps its refs in Git newer (3.0) reftable format
    ReftableNotSupported,

    /// Something else moved ref while we were changing it
    RefChanged {
        /// The ref like `refs/heads/main`
        name: String,
    },
}

/// `wik::Result<T>` is short for `Result<T, wik::Error>`
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Fs {
                action,
                path,
                source,
            } => write!(f, "cannot {action} {}: {source}", path.display()),
            Self::NotARepository => {
                write!(
                    f,
                    "not a git repository (or any of the parent directories): .git"
                )
            }
            Self::Unsupported { path, what } => {
                write!(
                    f,
                    "cannot store {}: {what} are not supported yet",
                    path.display()
                )
            }
            Self::InvalidObjectName { name } => {
                write!(f, "not a valid object name {name}")
            }
            Self::MissingVariable { name } => write!(f, "{name} is not set"),
            Self::InvalidDate { date } => {
                write!(f, "invalid date {date:?}: expected `<seconds> <+hhmm>`")
            }
            Self::MissingObject { id } => write!(f, "{id} is not a valid object"),
            Self::LockHeld { path } => write!(
                f,
                "unable to create '{}': File exists. If no other wik or git \
                 is running, delete this file",
                path.display()
            ),
            Self::BrokenRef { name } => write!(f, "{name} is a broken ref"),
            Self::ReftableNotSupported => write!(
                f,
                "this repository keeps its refs in the reftable format \
                 which wik does not support"
            ),
            Self::RefChanged { name } => {
                write!(f, "cannot update {name}: something else changed it")
            }
        }
    }
}

// make rust treat my error as rust error
impl std::error::Error for Error {}

// we need From so ? knows how to convert io::Error to our Error
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Adds .context(action, path) to io::Result, so we get more info for errors
pub(crate) trait IoResultExt<T> {
    fn context(self, action: &'static str, path: &Path) -> Result<T>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn context(self, action: &'static str, path: &Path) -> Result<T> {
        self.map_err(|source| Error::Fs {
            action,
            path: path.to_path_buf(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_mark_converts_io_errors() {
        fn read_missing() -> Result<Vec<u8>> {
            let data = std::fs::read("/bad/path")?;
            Ok(data)
        }

        let err = read_missing().unwrap_err();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn context_records_action_path_cause() {
        let path = Path::new("/bad/path");
        let err = std::fs::read(path).context("read", path).unwrap_err();

        match err {
            Error::Fs {
                action,
                path,
                source,
            } => {
                assert_eq!(action, "read");
                assert_eq!(path, Path::new("/bad/path"));
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("expected Error::Fs, got {other:?}"),
        }
    }
}
