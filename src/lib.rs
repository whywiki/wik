//! wik: git compatible Version Control System

mod commit;
mod database;
mod error;
mod hash;
mod identity;
mod lockfile;
mod object;
mod refs;
mod repository;
mod tree_builder;
mod workspace;

pub use commit::{CommitOutcome, commit};
pub use database::Database;
pub use error::{Error, Result};
pub use hash::{HashAlgorithm, Sha1Crate, Sha1Own};
pub use identity::{Time, author_and_committer, now, parse_date};
pub use lockfile::Lockfile;
pub use object::{Commit, Entry, Mode, Object, ObjectId, Signature, Tree, hash_file};
pub use refs::{Head, Refs};
pub use repository::{InitOutcome, find_git_dir, init};
pub use workspace::snapshot;
