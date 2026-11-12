//! Integration tests for `wik::init` (checked against real Git)

mod common;

use std::fs;

use tempfile::TempDir;
use wik::{Error, InitOutcome};

use common::git;

#[test]
fn creates_objects_refs_head() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");

    let outcome = wik::init(&repo).unwrap();

    let git_dir = fs::canonicalize(repo.join(".git")).unwrap();

    assert!(matches!(outcome, InitOutcome::Created(dir) if dir == git_dir));
    assert!(git_dir.join("objects").is_dir());
    assert!(git_dir.join("refs").is_dir());
    assert_eq!(
        fs::read_to_string(git_dir.join("HEAD")).unwrap(),
        "ref: refs/heads/main\n"
    );
}

#[test]
fn real_git_recognizes_the_repository() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");

    wik::init(&repo).unwrap();

    assert_eq!(git(&repo, &["rev-parse", "--git-dir"]), ".git\n");
    assert_eq!(git(&repo, &["symbolic-ref", "HEAD"]), "refs/heads/main\n");
}

#[test]
fn second_init_keeps_head_and_restores_missing_dirs() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();

    let git_dir = repo.join(".git");

    fs::write(git_dir.join("HEAD"), "ref: refs/heads/other\n").unwrap();
    fs::remove_dir(git_dir.join("objects")).unwrap();

    let outcome = wik::init(&repo).unwrap();

    assert!(matches!(outcome, InitOutcome::Reinitialized(_)));
    assert_eq!(
        fs::read_to_string(git_dir.join("HEAD")).unwrap(),
        "ref: refs/heads/other\n"
    );
    assert!(git_dir.join("objects").is_dir());
}

#[test]
fn git_dir_without_head_behaves_like_new() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");

    for dir in ["objects", "refs"] {
        fs::create_dir_all(repo.join(".git").join(dir)).unwrap();
    }

    let outcome = wik::init(&repo).unwrap();

    assert!(matches!(outcome, InitOutcome::Created(_)));
    assert_eq!(git(&repo, &["symbolic-ref", "HEAD"]), "refs/heads/main\n");
}

#[test]
fn reports_which_path_could_not_be_created() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("not-dir");
    fs::write(&file, "").unwrap();

    let err = wik::init(&file).unwrap_err();

    match err {
        Error::Fs { action, path, .. } => {
            assert_eq!(action, "mkdir");
            assert_eq!(path, file);
        }
        other => panic!("expected Error::Fs, got {other:?}"),
    }
}
