//! Tree objects: the list of entries (files, folders) of one
//! directory.

use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

use crate::ObjectId;

/// What kind of entry a tree holds.
///
/// Git has fixed set of modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Regular non-executable file
    File,
}

impl Mode {
    /// Mode as Git writes it inside a tree
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "100644",
        }
    }
}

/// One entry of a tree:
/// - name
/// - kind of entry
/// - id of tis content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// File name without any folder. Doesn't have to be valid UTF-8
    pub name: OsString,
    /// What kind of entry this is
    pub mode: Mode,
    /// id of the blob holding the file content
    pub id: ObjectId,
}

/// Tree object: the entries of one directory.
///
/// Entries are always sorted by name as Git requires. [`Tree::new`] is
/// the only way to create a `Tree` and it sorts them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    entries: Vec<Entry>,
}

impl Tree {
    /// Creates a tree from `entries` sorted by name the same way Git does
    pub fn new(mut entries: Vec<Entry>) -> Self {
        entries.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
        Self { entries }
    }

    /// Tree content in Git format (without the object header).
    ///
    /// Every entry is written as:
    ///
    /// ```text
    /// <mode> <name>\0<id>
    /// ```
    ///
    /// where `<mode>` is text like `100644`, `\0` is one zero byte
    /// that ends the name, `<id>` is the id as raw bytes (20 for SHA-1)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        for entry in &self.entries {
            bytes.extend_from_slice(entry.mode.as_str().as_bytes());
            bytes.push(b' ');
            bytes.extend_from_slice(entry.name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(entry.id.as_bytes());
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Object, Sha1Crate};

    /// An entry for a regular file named `name` holding `content`
    fn file(name: &str, content: &[u8]) -> Entry {
        Entry {
            name: name.into(),
            mode: Mode::File,
            id: Object::Blob(content.to_vec()).id(&Sha1Crate),
        }
    }

    #[test]
    fn entry_is_mode_name_zero_byte_and_raw_id() {
        let entry = file("hello.txt", b"hello world\n");
        let id = entry.id;

        let bytes = Tree::new(vec![entry]).to_bytes();

        let mut expected = b"100644 hello.txt\0".to_vec();
        expected.extend_from_slice(id.as_bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn entries_are_sorted_by_name() {
        let tree = Tree::new(vec![file("b", b"2"), file("a", b"1")]);

        assert!(tree.to_bytes().starts_with(b"100644 a\0"));
    }

    #[test]
    fn tree_id_matches_git() {
        let tree = Tree::new(vec![
            file("hello.txt", b"hello world\n"),
            file("empty.txt", b""),
        ]);

        // `git write-tree` for a folder with exactly these two files
        assert_eq!(
            Object::Tree(tree).id(&Sha1Crate).to_string(),
            "5d4cc1b4fb07085da4f9ee62de8974ca78ccdac0"
        );
    }
}
