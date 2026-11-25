//! Integration tests for `wik::Database` checked with `git cat-file`
//! and `git fsck`

use std::fs;
use std::os::unix::fs::PermissionsExt;

use tempfile::TempDir;
use wik::{Database, Object, Sha1Crate};

use crate::helpers::git;

#[test]
fn stores_object_git_can_read() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();

    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));

    let id = database
        .store(&Object::Blob(b"hello world\n".to_vec()))
        .unwrap();

    let hex = id.to_string();
    assert_eq!(hex, "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    assert!(
        repo.join(".git/objects/3b/18e512dba79e4c8300dd08aeb37f8e728b8dad")
            .is_file()
    );
    assert_eq!(git(&repo, &["cat-file", "-t", &hex]), "blob\n");
    assert_eq!(git(&repo, &["cat-file", "-p", &hex]), "hello world\n");
}

#[test]
fn stored_objects_pass_git_fsck() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    let every_byte: Vec<u8> = (0..=255).collect();

    for content in [Vec::new(), b"text\n".to_vec(), every_byte] {
        database.store(&Object::Blob(content)).unwrap();
    }

    // `git fsck` checks that every object decompresses and matches its id.
    git(&repo, &["fsck", "--strict"]);
    assert!(git(&repo, &["count-objects"]).starts_with("3 objects,"));
}

#[test]
fn storing_the_same_object_twice_leaves_one_file() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    let blob = Object::Blob(b"hello world\n".to_vec());

    let first = database.store(&blob).unwrap();
    let second = database.store(&blob).unwrap();

    assert_eq!(first, second);

    let files: Vec<_> = fs::read_dir(repo.join(".git/objects/3b"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(files, ["18e512dba79e4c8300dd08aeb37f8e728b8dad"]);
}

#[test]
fn objects_are_read_only_like_gits() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));

    database
        .store(&Object::Blob(b"hello world\n".to_vec()))
        .unwrap();

    let path = repo.join(".git/objects/3b/18e512dba79e4c8300dd08aeb37f8e728b8dad");
    let mode = fs::metadata(path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o444);
}

#[test]
fn contains_only_stored_objects() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));

    let stored = database
        .store(&Object::Blob(b"hello world\n".to_vec()))
        .unwrap();
    let not_stored = Object::Blob(b"never stored\n".to_vec()).id(&Sha1Crate);

    assert!(database.contains(stored));
    assert!(!database.contains(not_stored));
}
