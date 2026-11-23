//! Git objects and their on-disk byte format

use std::fs;
use std::path::Path;

use crate::error::IoResultExt;
use crate::{HashAlgorithm, ObjectId, Result};

/// Object that can be stored in the repository.
///
/// Git has a fixed set of object types, so this is an enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    /// The contents of one file
    Blob(Vec<u8>),
}

impl Object {
    /// The object in Git format: `<type> <length>\0` followed by the content.
    /// Later those bytes get hashed and stored.
    pub fn serialize(&self) -> Vec<u8> {
        let (kind, content): (&str, &[u8]) = match self {
            Self::Blob(data) => ("blob", data),
        };

        // header
        let mut bytes = format!("{kind} {}\0", content.len()).into_bytes();
        // content
        bytes.extend_from_slice(content);
        bytes
    }

    /// The id of this object when hashed with `hasher`
    pub fn id(&self, hasher: &dyn HashAlgorithm) -> ObjectId {
        hasher.hash(&self.serialize())
    }

    /// Reads the file at `path` into a blob
    ///
    /// # Errors
    ///
    /// Returns [`Error::Fs`](crate::Error::Fs) if the file cannot be read
    pub fn blob_from_file(path: &Path) -> Result<Self> {
        let content = fs::read(path).context("read", path)?;
        Ok(Self::Blob(content))
    }
}

/// Reads the file at `path` and returns the id it would have as a blob
/// (like `git hash-object <path>`). Nothing is written to the
/// repository.
///
/// # Errors
///
/// Returns [`Error::Fs`](crate::Error::Fs) if the file cannot be read.
pub fn hash_file(path: &Path, hasher: &dyn HashAlgorithm) -> Result<ObjectId> {
    Ok(Object::blob_from_file(path)?.id(hasher))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sha1Crate;

    #[test]
    fn blob_is_serialized_with_header() {
        let blob = Object::Blob(b"hello hash".to_vec());

        assert_eq!(blob.serialize(), b"blob 10\0hello hash");
    }

    #[test]
    fn blob_ids_match_git() {
        // checked with `git hash-object`
        let empty = Object::Blob(Vec::new());
        let hello = Object::Blob(b"hello hash\n".to_vec());

        assert_eq!(
            empty.id(&Sha1Crate).to_string(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
        assert_eq!(
            hello.id(&Sha1Crate).to_string(),
            "ae5df31bfbaedcb91461d14cb16ae798271fe13f"
        );
    }
}
