//! wik: git compatible Version Control System

mod database;
mod error;
mod hash;
mod object;
mod repository;
mod workspace;

pub use database::Database;
pub use error::{Error, Result};
pub use hash::{HashAlgorithm, Sha1Crate, Sha1Own};
pub use object::{Entry, Mode, Object, ObjectId, Tree, hash_file};
pub use repository::{InitOutcome, find_git_dir, init};
pub use workspace::snapshot;
