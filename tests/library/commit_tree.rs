//! Integration tests for `Database::store_commit` checked against
//! `git commit-tree` with the same tree, parents, people, times and message

use std::fs;

use tempfile::TempDir;
use wik::{Commit, Database, Error, Object, ObjectId, Sha1Crate};

use crate::helpers::{git, identity_var};

/// A commit of `tree` after `parents` made by the test identity.
fn commit(tree: ObjectId, parents: Vec<ObjectId>, message: &str) -> Commit {
    let (author, committer) =
        wik::author_and_committer(identity_var, wik::now()).expect("the test identity is complete");
    Commit {
        tree,
        parents,
        author,
        committer,
        message: message.to_string(),
    }
}

#[test]
fn commits_match_git_commit_tree() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    fs::create_dir(repo.join("src")).unwrap();
    fs::write(repo.join("README.md"), "read me\n").unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
    let tree = wik::snapshot(&repo, &database).unwrap();
    let tree_hex = tree.to_string();

    // A first commit then one with the first as its parent
    let first = database
        .store_commit(commit(tree, vec![], "first commit\n"))
        .unwrap();
    let second = database
        .store_commit(commit(tree, vec![first], "second commit\n"))
        .unwrap();

    let git_first = git(&repo, &["commit-tree", &tree_hex, "-m", "first commit"]);
    let first_hex = first.to_string();
    let git_second = git(
        &repo,
        &[
            "commit-tree",
            &tree_hex,
            "-p",
            &first_hex,
            "-m",
            "second commit",
        ],
    );
    assert_eq!(format!("{first}\n"), git_first);
    assert_eq!(format!("{second}\n"), git_second);

    assert_eq!(
        git(&repo, &["cat-file", "-t", &second.to_string()]),
        "commit\n"
    );
    git(&repo, &["fsck", "--strict"]);
}

#[test]
fn missing_tree_or_parent_is_an_error() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().join("repo");
    wik::init(&repo).unwrap();
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    let tree = database.store(&Object::Blob(b"stored\n".to_vec())).unwrap();
    let missing = Object::Blob(b"never stored\n".to_vec()).id(&Sha1Crate);

    let no_tree = database.store_commit(commit(missing, vec![], "x\n"));
    let no_parent = database.store_commit(commit(tree, vec![missing], "x\n"));

    assert!(matches!(no_tree, Err(Error::MissingObject { id }) if id == missing));
    assert!(matches!(no_parent, Err(Error::MissingObject { id }) if id == missing));
}
