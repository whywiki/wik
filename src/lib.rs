//! wik: git compatible Version Control System

mod error;
mod hash;
mod object_id;
mod repository;

pub use error::{Error, Result};
pub use hash::{HashAlgorithm, Sha1Crate};
pub use object_id::ObjectId;
pub use repository::{InitOutcome, init};
