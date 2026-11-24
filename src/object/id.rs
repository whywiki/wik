//! Object ids: the hash that names every object in the repository.

use std::fmt;
use std::str::FromStr;

use crate::Error;

/// The id of an object: the hash of its header and content.
///
/// One variant per hash format Git supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectId {
    /// A 20-byte SHA-1 hash
    Sha1([u8; 20]),
}

impl ObjectId {
    /// The raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Sha1(bytes) => bytes,
        }
    }
}

impl fmt::Display for ObjectId {
    /// Formats the id as lowercase hex (same as Git prints it)
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.as_bytes() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for ObjectId {
    type Err = Error;

    /// Reads an Git id written as hex
    ///
    /// It must be full id: 40 hex characters for SHA-1. Uppercase and
    /// lowercase letters are both accepted like in Git.
    fn from_str(hex: &str) -> Result<Self, Error> {
        let invalid = || Error::InvalidObjectName {
            name: hex.to_string(),
        };

        // Identify the hash
        // SHA-1
        if hex.len() != 40 {
            return Err(invalid());
        }

        // Every 2 hex characters make one byte
        let mut bytes = [0_u8; 20];
        let (pairs, _) = hex.as_bytes().as_chunks::<2>();
        for (byte, [high, low]) in bytes.iter_mut().zip(pairs) {
            let high = hex_digit(*high).ok_or_else(invalid)?;
            let low = hex_digit(*low).ok_or_else(invalid)?;
            *byte = high * 16 + low;
        }
        Ok(Self::Sha1(bytes))
    }
}

/// The value of one hex character (`0`-`9`, `a`-`f`, `A`-`F`) or `None`
/// if it isn't hex character.
fn hex_digit(character: u8) -> Option<u8> {
    match character {
        b'0'..=b'9' => Some(character - b'0'),
        b'a'..=b'f' => Some(character - b'a' + 10),
        b'A'..=b'F' => Some(character - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_as_lowercase_hex_with_leading_zeros() {
        // repeat 0a 20 times
        let id = ObjectId::Sha1([0x0a; 20]);

        assert_eq!(id.to_string(), "0a".repeat(20));
    }

    #[test]
    fn parses_what_it_displays() {
        let hex = "3b18e512dba79e4c8300dd08aeb37f8e728b8dad";

        let id: ObjectId = hex.parse().unwrap();

        assert_eq!(id.to_string(), hex);
    }

    #[test]
    fn accepts_uppercase_like_git() {
        let id: ObjectId = "3B18E512DBA79E4C8300DD08AEB37F8E728B8DAD".parse().unwrap();

        assert_eq!(id.to_string(), "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    }

    #[test]
    fn rejects_bad_names() {
        let bad = [
            // too short
            "3b18e5",
            // `g` is not hex
            "g b18e512dba79e4c8300dd08aeb37f8e728b8da",
            // `é` takes 2 of them
            "é18e512dba79e4c8300dd08aeb37f8e728b8dad",
        ];

        for name in bad {
            let result: Result<ObjectId, Error> = name.parse();
            assert!(
                matches!(result, Err(Error::InvalidObjectName { .. })),
                "{name:?} was accepted"
            );
        }
    }
}
