//! Making a commit of the working folder.
//!
//! It joins the pieces from the earlier steps:
//! 1. `snapshot` stores the files and gives a tree
//! 2. `Refs` tells which commit `HEAD` is on
//! 3. `store_commit` stores the new commit
//! 4. `update_ref` moves the branch to it

use std::path::Path;

use crate::{Commit, Database, Error, Head, ObjectId, Refs, Result, Signature, snapshot};

/// What [`commit`] did
#[derive(Debug)]
pub struct CommitOutcome {
    /// New commit
    pub id: ObjectId,
    /// What `HEAD` pointed to
    pub head: Head,
    /// Is this is a root commit: the first one with no parent
    pub root: bool,
}

/// Commits every file in `root` of the working folder of the repository in
/// `git_dir`. Parent is the commit that `HEAD` is on and the branch
/// (or a detached `HEAD`) moves to the new commit.
///
/// The message is stored as it is so the caller adds the newline at the
/// end.
///
/// # Errors
///
/// Returns [`Error::EmptyMessage`] if the message is only whitespace
/// [`Error::RefChanged`] if someone else moved the branch while we were
/// working and the errors of [`snapshot`] and [`Refs`].
pub fn commit(
    root: &Path,
    git_dir: &Path,
    database: &Database,
    author: Signature,
    committer: Signature,
    message: String,
) -> Result<CommitOutcome> {
    if message.trim().is_empty() {
        return Err(Error::EmptyMessage);
    }

    // Read where we are before storing anything. This stops early in
    // a repository with reftable or when `HEAD` is broken.
    let refs = Refs::new(git_dir)?;
    let head = refs.read_head()?;
    let parent = refs.read_ref(head.ref_name())?;

    let tree = snapshot(root, database)?;
    let parents = match parent {
        Some(id) => vec![id],
        None => Vec::new(),
    };
    let id = database.store_commit(Commit {
        tree,
        parents,
        author,
        committer,
        message,
    })?;

    // Moves the branch only if it is still at `parent` (see `update_ref`)
    refs.update_ref(head.ref_name(), id, parent)?;

    Ok(CommitOutcome {
        id,
        head,
        root: parent.is_none(),
    })
}
