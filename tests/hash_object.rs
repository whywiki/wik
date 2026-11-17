//! Integration tests for `wik::hash_file` checked against `git hash-object`

mod common;

use std::fs;

use tempfile::TempDir;
use wik::{Error, Sha1Crate};

use common::git;

#[test]
fn ids_match_git_hash_object() {
    let tmp = TempDir::new().unwrap();
    let every_byte: Vec<u8> = (0..=255).collect();
    let large: Vec<u8> = (0..1_000_000_u32).map(|i| (i % 251) as u8).collect();

    let cases: [(&str, &[u8]); 5] = [
        ("empty", b""),
        ("text", b"hello world\n"),
        ("no-newline", b"no newline at the end"),
        ("binary", &every_byte),
        ("large", &large),
    ];

    for (name, content) in cases {
        let path = tmp.path().join(name);
        fs::write(&path, content).unwrap();

        let wik_id = wik::hash_file(&path, &Sha1Crate).unwrap();
        let git_output = git(tmp.path(), &["hash-object", name]);
        let git_id = git_output.trim_end();

        assert_eq!(wik_id.to_string(), git_id, "different id for {name}");
    }
}

#[test]
fn missing_file_reports_its_path() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("missing");

    let err = wik::hash_file(&path, &Sha1Crate).unwrap_err();

    match err {
        Error::Fs {
            action, path: p, ..
        } => {
            assert_eq!(action, "read");
            assert_eq!(p, path);
        }
        other => panic!("expected Error::Fs, got {other:?}"),
    }
}
