//! End-to-end tests for `wik hash-object`

use std::fs;

use tempfile::TempDir;

use crate::helpers::{stderr, stdout, wik};

#[test]
fn hash_object_prints_one_id_per_file() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("empty"), "").unwrap();
    fs::write(tmp.path().join("hello"), "hello world\n").unwrap();

    let output = wik(tmp.path(), &["hash-object", "empty", "hello"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\n\
         3b18e512dba79e4c8300dd08aeb37f8e728b8dad\n"
    );
}

#[test]
fn hash_object_of_missing_file_is_fatal() {
    let tmp = TempDir::new().unwrap();

    let output = wik(tmp.path(), &["hash-object", "missing"]);

    assert_eq!(output.status.code(), Some(128));
    assert!(
        stderr(&output).starts_with("fatal: cannot read missing: "),
        "unexpected stderr: {}",
        stderr(&output)
    );
}
