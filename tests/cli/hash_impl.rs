//! End-to-end tests for the global `--hash-impl` flag

use std::fs;

use tempfile::TempDir;

use crate::helpers::{stderr, stdout, wik};

#[test]
fn hash_impl_own_gives_the_same_ids_as_the_default() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("empty"), "").unwrap();
    fs::write(tmp.path().join("hello"), "hello world\n").unwrap();

    let default = wik(tmp.path(), &["hash-object", "empty", "hello"]);
    let own = wik(
        tmp.path(),
        &["--hash-impl=own", "hash-object", "empty", "hello"],
    );

    assert_eq!(own.status.code(), Some(0));
    assert_eq!(stdout(&own), stdout(&default));
}

#[test]
fn hash_impl_can_also_come_after_the_command() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("hello"), "hello world\n").unwrap();

    let output = wik(tmp.path(), &["hash-object", "--hash-impl=own", "hello"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "3b18e512dba79e4c8300dd08aeb37f8e728b8dad\n"
    );
}

#[test]
fn unknown_hash_impl_is_rejected() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("hello"), "hello world\n").unwrap();

    let output = wik(tmp.path(), &["--hash-impl=onw", "hash-object", "hello"]);

    assert_ne!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("invalid value 'onw'"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}
