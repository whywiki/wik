//! Integration tests for `wik::commit` checked against `git commit` on
//! the same files with the same people times and messages

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wik::{CommitOutcome, Database, Error, Head, Refs, Sha1Crate};

use crate::helpers::{git, identity_var};

const MAIN: &str = "refs/heads/main";

/// Commits the files in the wik repository `repo` with the test identity
fn wik_commit(repo: &Path, message: &str) -> wik::Result<CommitOutcome> {
    let git_dir = repo.join(".git");
    let database = Database::new(&git_dir, Box::new(Sha1Crate));
    let (author, committer) =
        wik::author_and_committer(identity_var, wik::now()).expect("the test identity is complete");
    wik::commit(
        repo,
        &git_dir,
        &database,
        author,
        committer,
        message.to_string(),
    )
}

/// Commits the files in the Git repository `repo` with real Git and
/// returns the id of the new commit
fn git_commit(repo: &Path, message: &str) -> String {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-q", "-m", message]);
    git(repo, &["rev-parse", "HEAD"]).trim_end().to_string()
}

/// Two repositories in `tmp`: one made by wik and one made by Git.
fn twins(tmp: &TempDir) -> (PathBuf, PathBuf) {
    let wik_repo = tmp.path().join("wik");
    let git_repo = tmp.path().join("git");
    wik::init(&wik_repo).expect("wik init failed");
    git(tmp.path(), &["init", "-q", "-b", "main", "git"]);
    (wik_repo, git_repo)
}

/// Writes the same file into both repositories.
fn write_both(repos: [&Path; 2], file: &str, content: &str) {
    for repo in repos {
        let path = repo.join(file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create folder");
        }
        fs::write(path, content).expect("failed to write file");
    }
}

#[test]
fn commits_match_git_commit() {
    let tmp = TempDir::new().unwrap();
    let (wik_repo, git_repo) = twins(&tmp);
    let both = [wik_repo.as_path(), git_repo.as_path()];

    write_both(both, "README.md", "read me\n");
    write_both(both, "src/main.rs", "fn main() {}\n");
    let first = wik_commit(&wik_repo, "first commit\n").unwrap();
    assert_eq!(first.id.to_string(), git_commit(&git_repo, "first commit"));
    assert_eq!(first.head, Head::Branch(MAIN.to_string()));
    assert!(first.root);

    // Changed file and a new one. The parent must be the first commit
    write_both(both, "README.md", "read me again\n");
    write_both(both, "src/lib.rs", "pub fn wik() {}\n");
    let second = wik_commit(&wik_repo, "second commit\n").unwrap();
    assert_eq!(
        second.id.to_string(),
        git_commit(&git_repo, "second commit")
    );
    assert!(!second.root);

    // Real Git sees both commits on main in the wik repository
    assert_eq!(
        git(&wik_repo, &["log", "--format=%s", "main"]),
        "second commit\nfirst commit\n"
    );
    git(&wik_repo, &["fsck", "--strict"]);
}

#[test]
fn history_survives_packed_refs() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    fs::write(repo.join("a.txt"), "one\n").unwrap();
    let first = wik_commit(&repo, "first\n").unwrap();

    // `pack-refs` moves main into `packed-refs`. The next commit must
    // still find the first one as its parent
    git(&repo, &["pack-refs", "--all"]);
    assert!(!repo.join(".git/refs/heads/main").exists());
    fs::write(repo.join("a.txt"), "two\n").unwrap();
    let second = wik_commit(&repo, "second\n").unwrap();

    assert!(!second.root);
    assert_eq!(
        git(&repo, &["rev-parse", "main~1"]),
        format!("{}\n", first.id)
    );
    assert_eq!(
        git(&repo, &["rev-parse", "main"]),
        format!("{}\n", second.id)
    );
}

#[test]
fn detached_head_moves_itself() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    fs::write(repo.join("a.txt"), "one\n").unwrap();
    let first = wik_commit(&repo, "first\n").unwrap();

    git(
        &repo,
        &["update-ref", "--no-deref", "HEAD", &first.id.to_string()],
    );
    fs::write(repo.join("a.txt"), "two\n").unwrap();
    let second = wik_commit(&repo, "second\n").unwrap();

    assert_eq!(second.head, Head::Detached);
    assert_eq!(
        git(&repo, &["rev-parse", "HEAD~1"]),
        format!("{}\n", first.id)
    );
    // main stays where it was
    assert_eq!(
        git(&repo, &["rev-parse", "main"]),
        format!("{}\n", first.id)
    );
}

#[test]
fn empty_message_is_refused() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    fs::write(repo.join("a.txt"), "one\n").unwrap();

    let result = wik_commit(&repo, " \n\n");

    assert!(matches!(result, Err(Error::EmptyMessage)));
    let refs = Refs::new(&repo.join(".git")).unwrap();
    assert_eq!(refs.read_ref(MAIN).unwrap(), None);
}
