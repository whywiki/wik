//! wik: git compatible Version Control System

mod database;
mod error;
mod hash;
mod object;
mod repository;

pub use database::Database;
pub use error::{Error, Result};
pub use hash::{HashAlgorithm, Sha1Crate, Sha1Own};
pub use object::{Object, ObjectId, hash_file};
pub use repository::{InitOutcome, find_git_dir, init};
