//! End-to-end tests for `wik init`

use std::fs;

use tempfile::TempDir;

use crate::helpers::{stderr, stdout, wik};

#[test]
fn prints_where_the_repo_was_created() {
    let tmp = TempDir::new().unwrap();

    let output = wik(tmp.path(), &["init", "repo"]);

    let git_dir = fs::canonicalize(tmp.path().join("repo").join(".git")).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        format!(
            "Initialized empty Wik repository in {}/\n",
            git_dir.display()
        )
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn without_path_uses_current_dir() {
    let tmp = TempDir::new().unwrap();

    let output = wik(tmp.path(), &["init"]);

    let git_dir = fs::canonicalize(tmp.path().join(".git")).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        format!(
            "Initialized empty Wik repository in {}/\n",
            git_dir.display()
        )
    );
}

#[test]
fn second_init_prints_reinitialized() {
    let tmp = TempDir::new().unwrap();

    wik(tmp.path(), &["init", "repo"]);
    let output = wik(tmp.path(), &["init", "repo"]);

    let git_dir = fs::canonicalize(tmp.path().join("repo").join(".git")).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        format!(
            "Reinitialized existing Wik repository in {}/\n",
            git_dir.display()
        )
    );
}

#[test]
fn failure_prints_fatal_and_exits_with_128() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("not-a-dir"), "").unwrap();

    let output = wik(tmp.path(), &["init", "not-a-dir"]);

    assert_eq!(output.status.code(), Some(128));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).starts_with("fatal: cannot mkdir not-a-dir:"),
        "unexpected stderr: {}",
        stderr(&output)
    );
}
