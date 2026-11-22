//! Integration tests for `wik::find_git_dir` checked against
//! `git rev-parse --absolute-git-dir`

use std::fs;

use tempfile::TempDir;
use wik::Error;

use crate::helpers::git_with_ceiling;

#[test]
fn finds_the_repository_in_the_start_directory() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();

    let git_dir = wik::find_git_dir(&repo).unwrap();

    assert_eq!(git_dir, fs::canonicalize(repo.join(".git")).unwrap());
}

#[test]
fn finds_the_same_repository_as_git_from_a_subdirectory() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let deep = repo.join("a/b/c");
    fs::create_dir_all(&deep).unwrap();

    let ours = wik::find_git_dir(&deep).unwrap();
    let theirs = git_with_ceiling(&deep, &["rev-parse", "--absolute-git-dir"], tmp.path());

    assert_eq!(ours.to_str().unwrap(), theirs.trim_end());
}

#[test]
fn skips_a_git_dir_that_is_not_a_repository() {
    let tmp = TempDir::new().unwrap();
    let outer = tmp.path().join("outer");
    wik::init(&outer).unwrap();
    // `.git` without `HEAD` and `refs` is not accepted
    let inner = outer.join("inner");
    fs::create_dir_all(inner.join(".git/objects")).unwrap();

    let ours = wik::find_git_dir(&inner).unwrap();
    let theirs = git_with_ceiling(&inner, &["rev-parse", "--absolute-git-dir"], tmp.path());

    assert_eq!(ours.to_str().unwrap(), theirs.trim_end());
    assert_eq!(ours, fs::canonicalize(outer.join(".git")).unwrap());
}

#[test]
fn outside_a_repository_is_an_error() {
    let tmp = TempDir::new().unwrap();

    let err = wik::find_git_dir(tmp.path()).unwrap_err();

    assert!(matches!(err, Error::NotARepository), "got {err:?}");
}
