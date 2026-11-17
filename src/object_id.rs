//! Object ids: the hash that names every object in the repository.

use std::fmt;

/// the id of an object: the hash of its header and content.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_as_lowercase_hex_with_leading_zeros() {
        // repeat 0a, 20 times
        let id = ObjectId::Sha1([0x0a; 20]);

        assert_eq!(id.to_string(), "0a".repeat(20));
    }
}
