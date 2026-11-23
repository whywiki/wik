//! The working folder: the files a snapshot is made from.

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::error::IoResultExt;
use crate::repository::GIT_DIR;
use crate::{Database, Entry, Error, Mode, Object, ObjectId, Result, Tree};

/// Stores every file in the folder `root` as a blob, then create a tree out
/// of them and returns its tree id. `.git` is left out.
///
/// # Errors
///
/// Returns [`Error::Unsupported`] for a subfolder, and executable file,
/// a symbolic link or a special file, and [`Error::Fs`] if a file cannot
/// be read or an object cannot be written.
pub fn snapshot(root: &Path, database: &Database) -> Result<ObjectId> {
    let mut entries: Vec<Entry> = Vec::new();

    for name in list_files(root)? {
        let id = database.store(&Object::blob_from_file(&root.join(&name))?)?;
        entries.push(Entry {
            name,
            mode: Mode::File,
            id,
        });
    }

    database.store(&Object::Tree(Tree::new(entries)))
}

/// Gives the names of the files stored in `root`, without `.git` (no ordering)
fn list_files(root: &Path) -> Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();

    for entry in fs::read_dir(root).context("read", root)? {
        let entry = entry.context("read", root)?;
        let name = entry.file_name();

        if name == GIT_DIR {
            continue;
        }

        // check type of file and if we support it
        let path = entry.path();
        let metadata = entry.metadata().context("read", &path)?;
        if let Some(what) = unsupported(&metadata) {
            return Err(Error::Unsupported { path, what });
        }

        names.push(name);
    }

    Ok(names)
}

/// Check whether wik can store a file of this kind yet.
///
/// Returns `None` if it can or the name of the kind in plural if it can't.
fn unsupported(metadata: &fs::Metadata) -> Option<&'static str> {
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        Some("subdirectories")
    } else if file_type.is_symlink() {
        Some("symbolic links")
    } else if !file_type.is_file() {
        Some("special files")
    // Git treats a file as executable if its owner may execute it
    } else if metadata.permissions().mode() & 0o100 != 0 {
        Some("executable files")
    } else {
        None
    }
}
