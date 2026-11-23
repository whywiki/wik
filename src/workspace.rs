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
/// Returns [`Error::Unsupported`] for a subfolder,
/// a symbolic link or a special file, and [`Error::Fs`] if a file cannot
/// be read or an object cannot be written.
pub fn snapshot(root: &Path, database: &Database) -> Result<ObjectId> {
    let mut entries: Vec<Entry> = Vec::new();

    for (name, mode) in list_files(root)? {
        let id = database.store(&Object::blob_from_file(&root.join(&name))?)?;
        entries.push(Entry { name, mode, id });
    }

    database.store(&Object::Tree(Tree::new(entries)))
}

/// The files directly in `root` without `.git` with their modes.
/// The order is not sorted.
fn list_files(root: &Path) -> Result<Vec<(OsString, Mode)>> {
    let mut files: Vec<(OsString, Mode)> = Vec::new();

    for entry in fs::read_dir(root).context("read", root)? {
        let entry = entry.context("read", root)?;

        let name = entry.file_name();
        if name == GIT_DIR {
            continue;
        }

        // check type of file and if we support it
        let path = entry.path();
        let metadata = entry.metadata().context("read", &path)?;

        match file_mode(&metadata) {
            Ok(mode) => files.push((name, mode)),
            Err(what) => return Err(Error::Unsupported { path, what }),
        }
    }

    Ok(files)
}

/// Returns the tree mode for a file with this `metadata`.
///
/// If wik cannot store this kind of file yet it return its name instead
fn file_mode(metadata: &fs::Metadata) -> Result<Mode, &'static str> {
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        Err("subdirectories")
    } else if file_type.is_symlink() {
        Err("symbolic links")
    } else if !file_type.is_file() {
        Err("special files")
    // Git treats a file as executable if its owner may execute it
    } else if metadata.permissions().mode() & 0o100 != 0 {
        Ok(Mode::Executable)
    } else {
        Ok(Mode::File)
    }
}
