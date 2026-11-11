//! Creating repositories

use std::fs;
use std::path::{Path, PathBuf};

use crate::Result;

/// Name of the directory that holds git data
const GIT_DIR: &str = ".git";

/// Branch that `HEAD` points to in new repository
const DEFAULT_BRANCH: &str = "main";

/// Create a repository in `path` and returns absolute path for `.git` directory
///
/// It first creates a path for .git, next it creates:
/// - `.git/objects/`
/// - `.git/refs/`
/// - `.git/HEAD`
///
/// Those 3 are required to make Git recognize a directory as a Git repository
pub fn init(path: &Path) -> Result<PathBuf> {
    let git_dir = std::path::absolute(path)?.join(GIT_DIR);

    // create objects & refs
    for dir in ["objects", "refs"] {
        fs::create_dir_all(git_dir.join(dir))?;
    }

    // create HEAD
    let head = format!("ref: refs/heads/{DEFAULT_BRANCH}\n");
    fs::write(git_dir.join("HEAD"), head)?;

    Ok(fs::canonicalize(git_dir)?)
}
