//! Integration tests for `Refs` checked against real Git

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wik::{Commit, Database, Error, Head, Object, ObjectId, Refs, Sha1Crate, Tree};

use crate::helpers::{git, identity_var};

const MAIN: &str = "refs/heads/main";

/// New repository in `tmp` and its refs
fn repository(tmp: &TempDir) -> (PathBuf, Refs) {
    let repo = tmp.path().join("repo");
    wik::init(&repo).expect("failed to init");
    let refs = Refs::new(&repo.join(".git")).expect("failed to open refs");
    (repo, refs)
}

/// Stores a commit of the empty tree in `repo` and returns its id. It is
/// not on any branch.
fn commit(repo: &Path, message: &str) -> ObjectId {
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    let tree = database
        .store(&Object::Tree(Tree::new(Vec::new())))
        .expect("failed to store the empty tree");
    let (author, committer) =
        wik::author_and_committer(identity_var, wik::now()).expect("the test identity is complete");
    let commit = Commit {
        tree,
        parents: Vec::new(),
        author,
        committer,
        message: message.to_string(),
    };
    database
        .store_commit(commit)
        .expect("failed to store commit")
}

#[test]
fn new_repository_is_on_an_unborn_main() {
    let tmp = TempDir::new().unwrap();
    let (repo, refs) = repository(&tmp);

    assert_eq!(git(&repo, &["symbolic-ref", "HEAD"]), format!("{MAIN}\n"));
    assert_eq!(refs.read_head().unwrap(), Head::Branch(MAIN.to_string()));
    assert_eq!(refs.read_ref(MAIN).unwrap(), None);
}

#[test]
fn reads_loose_and_packed_refs() {
    let tmp = TempDir::new().unwrap();
    let (repo, refs) = repository(&tmp);
    let one = commit(&repo, "one\n");

    git(&repo, &["update-ref", MAIN, &one.to_string()]);
    assert_eq!(refs.read_ref(MAIN).unwrap(), Some(one));

    // `pack-refs --all` moves main into `packed-refs` and deletes the
    // loose file
    git(&repo, &["pack-refs", "--all"]);
    assert!(!repo.join(".git/refs/heads/main").exists());
    assert_eq!(refs.read_ref(MAIN).unwrap(), Some(one));
}

#[test]
fn updates_are_seen_by_git() {
    let tmp = TempDir::new().unwrap();
    let (repo, refs) = repository(&tmp);
    let one = commit(&repo, "one\n");
    let two = commit(&repo, "two\n");

    refs.update_ref(MAIN, one, None).unwrap();
    assert_eq!(git(&repo, &["rev-parse", "main"]), format!("{one}\n"));

    // After `pack-refs` the new id goes into a loose file again and it
    // wins over the old line in `packed-refs`.
    git(&repo, &["pack-refs", "--all"]);
    refs.update_ref(MAIN, two, Some(one)).unwrap();
    assert!(repo.join(".git/refs/heads/main").is_file());
    assert_eq!(git(&repo, &["rev-parse", "main"]), format!("{two}\n"));

    git(&repo, &["fsck", "--strict"]);
}

#[test]
fn update_is_refused_if_someone_else_moved_the_ref() {
    let tmp = TempDir::new().unwrap();
    let (repo, refs) = repository(&tmp);
    let one = commit(&repo, "one\n");
    let two = commit(&repo, "two\n");
    let three = commit(&repo, "three\n");
    refs.update_ref(MAIN, one, None).unwrap();

    // We read main then someone else moves it before we update it
    let seen = refs.read_ref(MAIN).unwrap();
    git(&repo, &["update-ref", MAIN, &two.to_string()]);
    let result = refs.update_ref(MAIN, three, seen);

    assert!(matches!(result, Err(Error::RefChanged { name }) if name == MAIN));
    assert_eq!(git(&repo, &["rev-parse", "main"]), format!("{two}\n"));
    assert!(!repo.join(".git/refs/heads/main.lock").exists());
}

#[test]
fn detached_head_is_moved_itself() {
    let tmp = TempDir::new().unwrap();
    let (repo, refs) = repository(&tmp);
    let one = commit(&repo, "one\n");
    let two = commit(&repo, "two\n");

    // `--no-deref` writes the id into HEAD itself and not into main
    git(
        &repo,
        &["update-ref", "--no-deref", "HEAD", &one.to_string()],
    );
    let head = refs.read_head().unwrap();
    assert_eq!(head, Head::Detached);
    assert_eq!(refs.read_ref(head.ref_name()).unwrap(), Some(one));

    refs.update_ref(head.ref_name(), two, Some(one)).unwrap();

    assert_eq!(git(&repo, &["rev-parse", "HEAD"]), format!("{two}\n"));
    assert_eq!(
        fs::read_to_string(repo.join(".git/HEAD")).unwrap(),
        format!("{two}\n")
    );
    assert_eq!(refs.read_ref(MAIN).unwrap(), None);
}

#[test]
fn broken_refs_are_errors() {
    let tmp = TempDir::new().unwrap();
    let (repo, refs) = repository(&tmp);

    fs::create_dir_all(repo.join(".git/refs/heads")).unwrap();
    fs::write(repo.join(".git/refs/heads/main"), "not an id\n").unwrap();
    let result = refs.read_ref(MAIN);
    assert!(matches!(result, Err(Error::BrokenRef { name }) if name == MAIN));

    // HEAD must not lead outside `refs/`.
    fs::write(repo.join(".git/HEAD"), "ref: refs/../../outside\n").unwrap();
    let result = refs.read_head();
    assert!(matches!(result, Err(Error::BrokenRef { name }) if name == "HEAD"));
}

#[test]
fn reftable_is_refused() {
    let tmp = TempDir::new().unwrap();
    let (repo, _) = repository(&tmp);

    // A reftable repository has this folder.
    fs::create_dir(repo.join(".git/reftable")).unwrap();

    let result = Refs::new(&repo.join(".git"));
    assert!(matches!(result, Err(Error::ReftableNotSupported)));
}
