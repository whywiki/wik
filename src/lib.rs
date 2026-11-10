//! wik: git compatible Version Control System

mod error;
mod repository;

pub use error::{Error, Result};
pub use repository::init;
