//! End-to-end tests for `wik commit-tree`.

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wik::{Database, Object, Sha1Crate, Tree};

use crate::helpers::{stderr, stdout, wik, wik_command, wik_with_input};

/// The id of the empty tree
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// `git commit-tree` of the empty tree with the test identity and the
/// message "first commit"
const FIRST_COMMIT: &str = "4cb646982ce6a8bd909378b40747de7a32c02630";

/// A repository in `tmp` that has the empty tree stored
fn repository(tmp: &TempDir) -> PathBuf {
    wik(tmp.path(), &["init", "repo"]);
    let repo = tmp.path().join("repo");
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    database
        .store(&Object::Tree(Tree::new(Vec::new())))
        .expect("failed to store the empty tree");
    repo
}

/// Is the object `hex` is stored in `repo`
fn stored(repo: &Path, hex: &str) -> bool {
    let (dir, file) = hex.split_at(2);
    repo.join(".git/objects").join(dir).join(file).is_file()
}

#[test]
fn prints_the_id_of_the_new_commit() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp);

    let output = wik(&repo, &["commit-tree", EMPTY_TREE, "-m", "first commit"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), format!("{FIRST_COMMIT}\n"));
    assert!(stored(&repo, FIRST_COMMIT));
}

#[test]
fn reads_the_message_from_stdin() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp);

    // `-m` adds a newline at the end stdin is used as it is
    let output = wik_with_input(&repo, &["commit-tree", EMPTY_TREE], b"first commit\n");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), format!("{FIRST_COMMIT}\n"));
}

#[test]
fn missing_tree_is_fatal() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp);
    let missing = "0123456789012345678901234567890123456789";

    let output = wik(&repo, &["commit-tree", missing, "-m", "x"]);

    assert_eq!(output.status.code(), Some(128));
    assert_eq!(
        stderr(&output),
        format!("fatal: {missing} is not a valid object\n")
    );
}

#[test]
fn missing_author_is_fatal() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp);

    let output = wik_command(&repo, &["commit-tree", EMPTY_TREE, "-m", "x"])
        .env_remove("GIT_AUTHOR_NAME")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(128));
    assert_eq!(stderr(&output), "fatal: GIT_AUTHOR_NAME is not set\n");
}

#[test]
fn invalid_id_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp);

    let output = wik(&repo, &["commit-tree", "1234", "-m", "x"]);

    assert_ne!(output.status.code(), Some(0));
    assert!(
        stderr(&output).contains("not a valid object name 1234"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}
