//! Shared wik error type

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

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
