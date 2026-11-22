//! wik: git compatible Version Control System

mod error;
mod hash;
mod object;
mod object_id;
mod repository;

pub use error::{Error, Result};
pub use hash::{HashAlgorithm, Sha1Crate, Sha1Own};
pub use object::{Object, hash_file};
pub use object_id::ObjectId;
pub use repository::{InitOutcome, find_git_dir, init};
