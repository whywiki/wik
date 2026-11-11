//! Creating repositories

use std::fs;
use std::path::{Path, PathBuf};

use crate::Result;
use crate::error::IoResultExt;

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
    // create path
    fs::create_dir_all(path).context("mkdir", path)?;

    // create .git directory
    let git_dir = path.join(GIT_DIR);
    fs::create_dir_all(&git_dir).context("mkdir", &git_dir)?;

    // create objects & refs
    for name in ["objects", "refs"] {
        let dir = git_dir.join(name);
        fs::create_dir_all(&dir).context("mkdir", &dir)?;
    }

    // create HEAD
    let head_path = git_dir.join("HEAD");
    let head_content = format!("ref: refs/heads/{DEFAULT_BRANCH}\n");
    fs::write(&head_path, head_content).context("write", &head_path)?;

    fs::canonicalize(&git_dir).context("resolve", &git_dir)
}
