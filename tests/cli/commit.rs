//! End-to-end tests for `wik commit`

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::helpers::{stderr, stdout, wik};

/// `git commit -m "first commit"` of one file `a.txt` with "one\n" in it
/// made with the test identity
const FIRST_COMMIT: &str = "795b6dc249363aaffb0dfce0bf86422f29ad8975";

/// A new repository `name` in `tmp`, with one file in it
fn repository(tmp: &TempDir, name: &str) -> PathBuf {
    wik(tmp.path(), &["init", name]);
    let repo = tmp.path().join(name);
    fs::write(repo.join("a.txt"), "one\n").expect("failed to write file");
    repo
}

/// The first 7 characters of the commit that `main` points to
fn main_short_id(repo: &Path) -> String {
    let id = fs::read_to_string(repo.join(".git/refs/heads/main")).expect("main is missing");
    id[..7].to_string()
}

#[test]
fn first_commit_is_a_root_commit() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp, "repo");

    let output = wik(&repo, &["commit", "-m", "first commit"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "[main (root-commit) 795b6dc] first commit\n"
    );
    let main = fs::read_to_string(repo.join(".git/refs/heads/main")).unwrap();
    assert_eq!(main, format!("{FIRST_COMMIT}\n"));
}

#[test]
fn next_commit_is_not_a_root_commit() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp, "repo");
    wik(&repo, &["commit", "-m", "first commit"]);
    fs::write(repo.join("a.txt"), "two\n").unwrap();

    let output = wik(&repo, &["commit", "-m", "second\ncommit\n\nmore text"]);

    assert_eq!(output.status.code(), Some(0));
    let short_id = main_short_id(&repo);
    assert_eq!(
        stdout(&output),
        format!("[main {short_id}] second commit\n")
    );
}

#[test]
fn works_from_a_subfolder() {
    let tmp = TempDir::new().unwrap();
    // Two repositories with the same files. One commits from its top
    // folder the other from a subfolder. Both must commit all files
    let top = repository(&tmp, "top");
    let sub = repository(&tmp, "sub");
    for repo in [&top, &sub] {
        fs::create_dir(repo.join("src")).unwrap();
        fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
    }

    wik(&top, &["commit", "-m", "first commit"]);
    let output = wik(&sub.join("src"), &["commit", "-m", "first commit"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(main_short_id(&sub), main_short_id(&top));
}

#[test]
fn detached_head_is_shown() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp, "repo");
    wik(&repo, &["commit", "-m", "first commit"]);
    // Detach HEAD: write the commit id into HEAD itself
    let main = fs::read_to_string(repo.join(".git/refs/heads/main")).unwrap();
    fs::write(repo.join(".git/HEAD"), &main).unwrap();
    fs::write(repo.join("a.txt"), "two\n").unwrap();

    let output = wik(&repo, &["commit", "-m", "second commit"]);

    assert_eq!(output.status.code(), Some(0));
    let head = fs::read_to_string(repo.join(".git/HEAD")).unwrap();
    assert_eq!(
        stdout(&output),
        format!("[detached HEAD {}] second commit\n", &head[..7])
    );
}

#[test]
fn empty_message_is_fatal() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp, "repo");

    let output = wik(&repo, &["commit", "-m", "  "]);

    assert_eq!(output.status.code(), Some(128));
    assert_eq!(
        stderr(&output),
        "fatal: Aborting commit due to empty commit message.\n"
    );
}

#[test]
fn message_is_required() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp, "repo");

    let output = wik(&repo, &["commit"]);

    assert_ne!(output.status.code(), Some(0));
    assert!(!repo.join(".git/refs/heads/main").exists());
}

#[test]
fn held_lock_is_fatal() {
    let tmp = TempDir::new().unwrap();
    let repo = repository(&tmp, "repo");
    // Someone else is changing main right now
    let lock = repo.join(".git/refs/heads/main.lock");
    fs::create_dir_all(lock.parent().unwrap()).unwrap();
    fs::write(&lock, "").unwrap();

    let output = wik(&repo, &["commit", "-m", "first commit"]);

    assert_eq!(output.status.code(), Some(128));
    assert!(
        stderr(&output).starts_with("fatal: unable to create '"),
        "unexpected stderr: {}",
        stderr(&output)
    );
    // Their lock is still there, and main was not created.
    assert!(lock.exists());
    assert!(!repo.join(".git/refs/heads/main").exists());
}
