//! Integration tests for `wik::snapshot` checked against
//! `git add -A && git write-tree` on the same folder

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use wik::{Database, Error, Sha1Crate};

use crate::helpers::git;

/// Creates a repository in `tmp` and returns its working folder and its
/// object database.
fn repository(tmp: &TempDir) -> (PathBuf, Database) {
    let repo = tmp.path().join("repo");
    wik::init(&repo).expect("wik init failed");
    let database = Database::new(&repo.join(".git"), Box::new(Sha1Crate));
    (repo, database)
}

/// The tree id real Git calculates for the files in `repo`.
fn git_write_tree(repo: &Path) -> String {
    git(repo, &["add", "-A"]);
    git(repo, &["write-tree"]).trim_end().to_string()
}

#[test]
fn tree_id_matches_git_write_tree() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);
    fs::write(repo.join("hello.txt"), "hello world\n").unwrap();
    fs::write(repo.join("empty.txt"), "").unwrap();
    fs::write(repo.join("a.txt"), "a\n").unwrap();
    fs::write(repo.join("Z.txt"), "Z\n").unwrap();

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
    git(&repo, &["fsck", "--strict"]);
}

#[test]
#[cfg_attr(target_os = "macos", ignore = "APFS only accepts UTF-8 file names")]
fn names_do_not_have_to_be_valid_utf8() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);

    // "déjà vu" in Latin-1 - not valid UTF-8
    let name = OsStr::from_bytes(b"d\xe9j\xe0 vu");
    fs::write(repo.join(name), "seen it before\n").unwrap();

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
}

#[test]
fn empty_folder_gives_the_empty_tree() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
}

/// Asserts that a snapshot of `repo` fails because of `path`, a `kind`
/// of file wik cannot store yet.
fn assert_unsupported(repo: &Path, database: &Database, path: &Path, kind: &str) {
    match wik::snapshot(repo, database) {
        Err(Error::Unsupported { path: p, what }) => {
            assert_eq!(what, kind);
            assert_eq!(p, path);
        }
        other => panic!("expected Error::Unsupported, got {other:?}"),
    }
}

#[test]
fn nested_folders_match_git() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);
    fs::create_dir_all(repo.join("src/hash")).unwrap();
    fs::write(repo.join("README.md"), "read me\n").unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(repo.join("src/hash/sha1.rs"), "pub fn hash() {}\n").unwrap();
    fs::create_dir(repo.join("notes")).unwrap();
    fs::write(repo.join("notes/todo.md"), "buy milk\n").unwrap();
    fs::write(repo.join("notes.txt"), "short note\n").unwrap();
    fs::write(repo.join("notes-old"), "old note\n").unwrap();

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
    git(&repo, &["fsck", "--strict"]);
}

#[test]
fn empty_folders_are_left_out_like_in_git() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);
    fs::write(repo.join("README.md"), "read me\n").unwrap();
    fs::create_dir_all(repo.join("empty/inside/too")).unwrap();

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
}

#[test]
fn nested_repository_is_unsupported() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);
    let inner = repo.join("vendor/lib");
    wik::init(&inner).unwrap();

    assert_unsupported(&repo, &database, &inner, "nested repositories");
}

#[test]
fn executable_files_match_git() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);
    let files = [
        ("plain.txt", 0o644),
        ("run.sh", 0o755),
        // Only the owner may execute: Git still counts it as executable
        ("owner.sh", 0o744),
        // Only the group may execute: Git does not count it as executable
        ("group.sh", 0o654),
    ];
    for (name, mode) in files {
        let path = repo.join(name);
        fs::write(&path, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
    }

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
}

#[test]
fn symbolic_links_match_git() {
    let tmp = TempDir::new().unwrap();
    let (repo, database) = repository(&tmp);
    fs::create_dir_all(repo.join("docs")).unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("README.md"), "read me\n").unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
    // A link to a file
    symlink("../README.md", repo.join("docs/README.md")).unwrap();
    // A link to a folder: stored as a link not followed into
    symlink("src", repo.join("code")).unwrap();
    // A broken link: the target does not exist
    symlink("missing.txt", repo.join("gone")).unwrap();

    let ours = wik::snapshot(&repo, &database).unwrap();

    assert_eq!(ours.to_string(), git_write_tree(&repo));
    git(&repo, &["fsck", "--strict"]);
}
