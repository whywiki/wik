//! Creating repositories

use std::path::{Path, PathBuf};

use crate::Result;

/// Create a repo in `path` and returns absolute path for `.git` directory
pub fn init(path: &Path) -> Result<PathBuf> {
    let root = std::path::absolute(path)?;
    Ok(root.join(".git"))
}
