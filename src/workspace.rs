//! The working folder: the files a snapshot is made from.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::repository::GIT_DIR;
use crate::tree_builder::store_tree;
use crate::{Database, Error, Mode, Object, ObjectId, Result};

/// Stores every file in the folder `root` as a blob, then create one tree per
/// folder and returns its tree id. `.git` is left out.
///
/// # Errors
///
/// Returns [`Error::Unsupported`] for a special file or nested repository,
/// and [`Error::Fs`] if a file cannot be read or an object cannot be written.
pub fn snapshot(root: &Path, database: &Database) -> Result<ObjectId> {
    let mut files = Vec::new();
    for (path, mode) in list_files(root)? {
        let full_path = root.join(&path);

        // link is stored as its target path so it is never followed
        let blob = if mode == Mode::Symlink {
            Object::blob_from_link(&full_path)?
        } else {
            Object::blob_from_file(&full_path)?
        };

        let id = database.store(&blob)?;
        files.push((path, mode, id));
    }
    store_tree(files, database)
}

/// Lists all files under `root` (also files in subfolders with their modes).
///
/// Paths are relative to `root`. `.git` is left out. The list is not sorted.
fn list_files(root: &Path) -> Result<Vec<(PathBuf, Mode)>> {
    let mut files = Vec::new();
    list_folder(root, Path::new(""), &mut files)?;
    Ok(files)
}

/// Adds the files in the folder `root/folder` to `files`.
///
/// For each subfolder it calls itself so files in subfolders are added
/// too. `folder` is relative to `root` and empty for `root` itself.
fn list_folder(root: &Path, folder: &Path, files: &mut Vec<(PathBuf, Mode)>) -> Result<()> {
    let dir = root.join(folder);

    for entry in fs::read_dir(&dir).context("read", &dir)? {
        let entry = entry.context("read", &dir)?;
        let name = entry.file_name();
        if name == GIT_DIR {
            // `.git` in the root is the our repository. Anywhere else it
            // means a nested repository
            if folder.as_os_str().is_empty() {
                continue;
            }
            return Err(Error::Unsupported {
                path: dir,
                what: "nested repositories",
            });
        }

        // The path of this entry relative to root
        let path = folder.join(&name);
        let metadata = entry.metadata().context("read", &entry.path())?;

        match file_mode(&metadata) {
            // A subfolder: list its files too
            Ok(Mode::Directory) => list_folder(root, &path, files)?,
            // A file: add it to the list.
            Ok(mode) => files.push((path, mode)),
            // Something wik cannot store yet
            Err(what) => {
                return Err(Error::Unsupported {
                    path: entry.path(),
                    what,
                });
            }
        }
    }
    Ok(())
}

/// Returns the tree mode for a file with this `metadata`.
///
/// If wik cannot store this kind of file yet it return its name instead
fn file_mode(metadata: &fs::Metadata) -> Result<Mode, &'static str> {
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        Ok(Mode::Directory)
    } else if file_type.is_symlink() {
        Ok(Mode::Symlink)
    } else if !file_type.is_file() {
        Err("special files")
    } else if metadata.permissions().mode() & 0o100 != 0 {
        // Git treats a file as executable if its owner may execute it
        Ok(Mode::Executable)
    } else {
        Ok(Mode::File)
    }
}
