//! Refs: names that point to commits like the branch
//! `refs/heads/main and `HEAD` which says which branch we are on.
//!
//! wik keeps them the same way Git does:
//! - ref is a small file like `.git/refs/heads/main` that holds commit id
//! - `git gc` can move refs into one file `.git/packed-refs`
//! - if a ref is in both places the loose file wins

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::{Error, Lockfile, ObjectId, Result};

/// What `HEAD` points to.
#[derive(Debug, PartialEq, Eq)]
pub enum Head {
    /// We are on a branch. It holds the full name like `refs/heads/main`
    Branch(String),
    /// `HEAD` holds a commit id itself not the name of a branch
    Detached,
}

impl Head {
    /// Ref that new commit moves: the branch or `HEAD` itself when
    /// it is detached.
    pub fn ref_name(&self) -> &str {
        match self {
            Self::Branch(name) => name,
            Self::Detached => "HEAD",
        }
    }
}

/// Refs of one repository
#[derive(Debug)]
pub struct Refs {
    git_dir: PathBuf,
}

impl Refs {
    /// Opens the refs of the repository in `git_dir`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ReftableNotSupported`] if the repository keeps its
    /// refs in Git newer reftable format.
    pub fn new(git_dir: &Path) -> Result<Self> {
        // Reftable repository has this folder
        if git_dir.join("reftable").exists() {
            return Err(Error::ReftableNotSupported);
        }

        Ok(Self {
            git_dir: git_dir.to_path_buf(),
        })
    }

    /// Reads what `HEAD` points to.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BrokenRef`] if `HEAD` points to a branch outside
    /// `refs/`, and [`Error::Fs`] if `HEAD` cannot be read.
    pub fn read_head(&self) -> Result<Head> {
        let path = self.git_dir.join("HEAD");
        let content = fs::read_to_string(&path).context("read", &path)?;

        // `ref: refs/heads/main` means we are on a branch. Anything else
        // should be a commit id which `read_ref("HEAD")` checks.
        match content.trim_end().strip_prefix("ref: ") {
            Some(name) if is_safe_ref_name(name) => Ok(Head::Branch(name.to_string())),
            Some(_) => Err(Error::BrokenRef {
                name: "HEAD".to_string(),
            }),
            None => Ok(Head::Detached),
        }
    }

    /// Reads the id that the ref `name` points to like `refs/heads/main`
    /// or `HEAD`. It looks at the loose file first then in `packed-refs`.
    ///
    /// `None` means the ref does not exist yet like `main` before the
    /// first commit.
    ///
    /// # Errors
    ///
    /// Returns [`Error::BrokenRef`] if the ref holds something that is not
    /// an id and [`Error::Fs`] if a file cannot be read.
    pub fn read_ref(&self, name: &str) -> Result<Option<ObjectId>> {
        let path = self.git_dir.join(name);

        match fs::read_to_string(&path) {
            Ok(content) => parse_id(name, content.trim_end()).map(Some),
            Err(err) if err.kind() == io::ErrorKind::NotFound => self.read_packed_ref(name),
            Err(source) => Err(Error::Fs {
                action: "read",
                path,
                source,
            }),
        }
    }

    /// Looks for the ref `name` in `packed-refs`.
    fn read_packed_ref(&self, name: &str) -> Result<Option<ObjectId>> {
        let path = self.git_dir.join("packed-refs");
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(Error::Fs {
                    action: "read",
                    path,
                    source,
                });
            }
        };

        // Each line is `<id> <name>`
        for line in content.lines() {
            if let Some((id, ref_name)) = line.split_once(' ')
                && ref_name == name
            {
                return parse_id(name, id).map(Some);
            }
        }
        Ok(None)
    }

    /// Moves the ref `name` to `new` but only if it still points to
    /// `expected`. `None` means it must not exist yet.
    ///
    /// The new id is always written to the loose file. If the ref is also
    /// in `packed-refs` that old line stays and the loose file wins.
    ///
    /// # Errors
    ///
    /// Returns [`Error::RefChanged`] if the ref no longer points to
    /// `expected` [`Error::LockHeld`] if someone else is changing it right
    /// now and [`Error::Fs`] if a file cannot be read or written.
    pub fn update_ref(&self, name: &str, new: ObjectId, expected: Option<ObjectId>) -> Result<()> {
        let path = self.git_dir.join(name);
        let mut lock = Lockfile::acquire(&path)?;

        // Someone may have moved the ref after the caller read it
        if self.read_ref(name)? != expected {
            return Err(Error::RefChanged {
                name: name.to_string(),
            });
        }

        lock.write(format!("{new}\n").as_bytes())?;
        lock.commit()
    }
}

/// Reads the id stored in the ref `name`.
fn parse_id(name: &str, text: &str) -> Result<ObjectId> {
    match text.parse() {
        Ok(id) => Ok(id),
        Err(_) => Err(Error::BrokenRef {
            name: name.to_string(),
        }),
    }
}

/// Whether `name` is safe to use as a path inside `.git`: it starts with
/// `refs/` and has no empty `.` or `..` parts.
fn is_safe_ref_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("refs/") else {
        return false;
    };
    for part in rest.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_ref_names() {
        assert!(is_safe_ref_name("refs/heads/main"));
        assert!(is_safe_ref_name("refs/heads/feature/login"));

        assert!(!is_safe_ref_name("HEAD"));
        assert!(!is_safe_ref_name("heads/main"));
        assert!(!is_safe_ref_name("refs/"));
        assert!(!is_safe_ref_name("refs/heads//main"));
        assert!(!is_safe_ref_name("refs/heads/./main"));
        assert!(!is_safe_ref_name("refs/../../outside"));
    }
}
