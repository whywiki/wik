//! Locks file the same way Git does
//!
//! To change `main` we first create `main.lock`. Only one process can
//! create it so only one process can change `main` at a time. We write
//! the new content into `main.lock` then rename it to `main`. The rename
//! happens in one step so nobody ever sees a half-written `main`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::{Error, Result};

/// Lock held on a file. The new content is written into `<file>.lock`
/// and replaces the file on [`commit`](Lockfile::commit).
#[derive(Debug)]
pub struct Lockfile {
    /// File we want to change
    path: PathBuf,
    /// `path` with `.lock`
    lock_path: PathBuf,
    /// Open `.lock` file
    file: File,
    /// Set to `true` after `commit` renamed the `.lock` file to the real
    /// file. Then `drop` cannot delete `.lock`
    committed: bool,
}

impl Lockfile {
    /// Locks `path` by creating `<path>.lock`. Missing parent folders are
    /// created.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LockHeld`] if the `.lock` file already exists
    /// which means someone else is changing the file. Returns
    /// [`Error::Fs`] if a folder or the `.lock` file cannot be created.
    pub fn acquire(path: &Path) -> Result<Self> {
        // `main` becomes `main.lock`
        let mut lock_name = path.as_os_str().to_owned();
        lock_name.push(".lock");
        let lock_path = PathBuf::from(lock_name);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("mkdir", parent)?;
        }

        // `create_new` fails if the file already exists. The system checks
        // and creates in one step, so if two processes try at the same
        // time exactly one of them gets the lock.
        let file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                return Err(Error::LockHeld { path: lock_path });
            }
            Err(source) => {
                return Err(Error::Fs {
                    action: "create",
                    path: lock_path,
                    source,
                });
            }
        };

        Ok(Self {
            path: path.to_path_buf(),
            lock_path,
            file,
            committed: false,
        })
    }

    /// Adds `data` to the new content.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Fs`] if the `.lock` file cannot be written.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.file.write_all(data).context("write", &self.lock_path)
    }

    /// Replaces the file with the new content and releases the lock.
    ///
    /// It takes `self` so the lock cannot be used after this.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Fs`] if the `.lock` file cannot be renamed. Then
    /// the lock is removed and the file stays as it was.
    pub fn commit(mut self) -> Result<()> {
        fs::rename(&self.lock_path, &self.path).context("rename", &self.lock_path)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for Lockfile {
    /// Deletes the `.lock` file if `commit` did not happen so a failed
    /// change does not block the next one.
    fn drop(&mut self) {
        if !self.committed {
            // We cannot report an error from `drop`, so the result is
            // ignored. The worst case is a leftover `.lock` file.
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn commit_replaces_the_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("main");
        fs::write(&path, "old\n").unwrap();

        let mut lock = Lockfile::acquire(&path).unwrap();
        lock.write(b"new").unwrap();
        lock.write(b"\n").unwrap();
        // Until the commit the file still has the old content
        assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
        lock.commit().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
        assert!(!tmp.path().join("main.lock").exists());
    }

    #[test]
    fn only_one_lock_at_a_time() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("main");

        let _first = Lockfile::acquire(&path).unwrap();
        let second = Lockfile::acquire(&path);
        assert!(
            matches!(second, Err(Error::LockHeld { path }) if path == tmp.path().join("main.lock"))
        );
    }

    #[test]
    fn dropping_without_commit_removes_the_lock() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("main");
        fs::write(&path, "old\n").unwrap();

        let mut lock = Lockfile::acquire(&path).unwrap();
        lock.write(b"new\n").unwrap();
        drop(lock);

        assert_eq!(fs::read_to_string(&path).unwrap(), "old\n");
        assert!(!tmp.path().join("main.lock").exists());
    }

    #[test]
    fn someone_elses_lock_is_left_alone() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("other");
        let lock_path = tmp.path().join("other.lock");
        fs::write(&lock_path, "not ours\n").unwrap();

        let result = Lockfile::acquire(&path);

        assert!(matches!(result, Err(Error::LockHeld { .. })));
        assert_eq!(fs::read_to_string(&lock_path).unwrap(), "not ours\n");
    }

    #[test]
    fn creates_missing_folders() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("refs/heads/main");

        let mut lock = Lockfile::acquire(&path).unwrap();
        lock.write(b"new\n").unwrap();
        lock.commit().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
    }
}
