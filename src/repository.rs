//! Creating repositories

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::Result;
use crate::error::IoResultExt;

/// Name of the directory that holds git data
const GIT_DIR: &str = ".git";

/// Branch that `HEAD` points to in new repository
const DEFAULT_BRANCH: &str = "main";

/// What [`init`] did
#[derive(Debug)]
pub enum InitOutcome {
    /// New repository was created
    Created(PathBuf),

    /// `.git/HEAD` already existed so it was left untouched
    Reinitialized(PathBuf),
}

/// Create a repository in `path` or reinitializes an existing one.
///
/// It first creates a `path` itself if needed, then `.git` and inside it:
/// - `.git/objects/`
/// - `.git/refs/`
/// - `.git/HEAD` (only if it doesn't exist yet)
///
/// Those 3 are required to make Git recognize a directory as a Git repository
///
/// Returns [`InitOutcome::Created`] or [`InitOutcome::Reinitialized`] with
/// absolute path of the `.git` directory
///
/// # Errors
///
/// Returns [`Error::Fs`](crate::Error::Fs) if directory cannot be created,
/// `HEAD` cannot be written, or path cannot be resolved
pub fn init(path: &Path) -> Result<InitOutcome> {
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

    let git_dir = fs::canonicalize(&git_dir).context("resolve", &git_dir)?;

    // create HEAD unless it alraedy exists
    let head_path = git_dir.join("HEAD");
    match File::create_new(&head_path) {
        Ok(mut file) => {
            let head_content = format!("ref: refs/heads/{DEFAULT_BRANCH}\n");
            file.write_all(head_content.as_bytes())
                .context("write", &head_path)?;
            Ok(InitOutcome::Created(git_dir))
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            Ok(InitOutcome::Reinitialized(git_dir))
        }
        Err(err) => Err(err).context("create", &head_path),
    }
}
