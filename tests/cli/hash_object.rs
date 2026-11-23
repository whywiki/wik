//! End-to-end tests for `wik hash-object`

use std::fs;

use tempfile::TempDir;

use crate::helpers::{stderr, stdout, wik};

#[test]
fn prints_one_id_per_file() {
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
fn missing_file_is_fatal() {
    let tmp = TempDir::new().unwrap();

    let output = wik(tmp.path(), &["hash-object", "missing"]);

    assert_eq!(output.status.code(), Some(128));
    assert!(
        stderr(&output).starts_with("fatal: cannot read missing: "),
        "unexpected stderr: {}",
        stderr(&output)
    );
}

#[test]
fn write_flag_stores_the_object_in_the_repository() {
    let tmp = TempDir::new().unwrap();
    wik(tmp.path(), &["init", "repo"]);
    let repo = tmp.path().join("repo");
    fs::write(repo.join("hello"), "hello world\n").unwrap();

    let output = wik(&repo, &["hash-object", "-w", "hello"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "3b18e512dba79e4c8300dd08aeb37f8e728b8dad\n"
    );
    assert!(
        repo.join(".git/objects/3b/18e512dba79e4c8300dd08aeb37f8e728b8dad")
            .is_file()
    );
}

#[test]
fn write_flag_finds_the_repository_from_a_subdirectory() {
    let tmp = TempDir::new().unwrap();
    wik(tmp.path(), &["init", "repo"]);
    let repo = tmp.path().join("repo");
    let sub = repo.join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("hello"), "hello world\n").unwrap();

    let output = wik(&sub, &["hash-object", "-w", "hello"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(
        repo.join(".git/objects/3b/18e512dba79e4c8300dd08aeb37f8e728b8dad")
            .is_file()
    );
}

#[test]
fn write_flag_outside_a_repository_is_fatal() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("hello"), "hello world\n").unwrap();

    let output = wik(tmp.path(), &["hash-object", "-w", "hello"]);

    assert_eq!(output.status.code(), Some(128));
    assert_eq!(
        stderr(&output),
        "fatal: not a git repository (or any of the parent directories): .git\n"
    );
}
