//! Shared wik error type

use std::fmt;
use std::io;

/// `wik::Error` is shared wik error type
#[derive(Debug)]
pub enum Error {
    /// IO failrues like missing file, permission denied, etc
    Io(io::Error),
}

/// `wik::Result<T>` is short for `Result<T, wik::Error>`
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

// make rust treat my error as rust error
impl std::error::Error for Error {}

// we need From so ? knows how to convert io:Error to our Error
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn question_mark_conversts_io_errors() {
        fn read_missing() -> Result<Vec<u8>> {
            let data = std::fs::read("bad/path")?;
            Ok(data)
        }

        let err = read_missing().unwrap_err();
        assert!(matches!(err, Error::Io(_)));
    }
}
