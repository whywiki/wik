//! Hash functions that turn bytes into object ids.
mod sha1_own;

use sha1::Digest;

use crate::ObjectId;

/// A hash function that produces object ids.
///
/// Implementations only hash bytes and they know nothing about Git.
/// The object header (`blob 6\0`) is added by the caller before hashing.
pub trait HashAlgorithm {
    /// Hashes `data` and returns the result as an object id
    fn hash(&self, data: &[u8]) -> ObjectId;
}

/// SHA-1 computed by the `sha1` crate.
#[derive(Debug, Clone, Copy)]
pub struct Sha1Crate;

impl HashAlgorithm for Sha1Crate {
    fn hash(&self, data: &[u8]) -> ObjectId {
        ObjectId::Sha1(sha1::Sha1::digest(data).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_crate_matches_known_values() {
        // Test vectors from the SHA-1 standard
        assert_eq!(
            Sha1Crate.hash(b"").to_string(),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            Sha1Crate.hash(b"abc").to_string(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }
}
