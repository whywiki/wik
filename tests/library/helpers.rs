//! Helpers for the library integration tests

use std::path::Path;
use std::process::Command;

/// Runs real Git with `args` inside `dir`. It asserts that the command
/// succeeded and returns what it printed to stdout.
///
/// It ignores the user and system config and it never searches for a repo
/// above `dir` and it uses [`IDENTITY`] as author and committer.
pub fn git(dir: &Path, args: &[&str]) -> String {
    git_with_ceiling(dir, args, dir.parent().unwrap_or(dir))
}

/// Like [`git`], but Git may search for a repository upward from `dir`
/// until it reaches `ceiling` (it does not look in `ceiling` itself)
pub fn git_with_ceiling(dir: &Path, args: &[&str], ceiling: &Path) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        // git ignores user configs
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        // git will not search for repo above this dir
        .env("GIT_CEILING_DIRECTORIES", ceiling)
        // the same author and committer as wik uses in the tests
        .envs(IDENTITY)
        .output()
        .expect("failed to start git. Is it installed?");

    assert!(
        output.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("git printed invalid UTF-8")
}

/// Author and committer used by every test for both Git and wik.
pub const IDENTITY: [(&str, &str); 6] = [
    ("GIT_AUTHOR_NAME", "Wiki"),
    ("GIT_AUTHOR_EMAIL", "wiki@example.com"),
    ("GIT_AUTHOR_DATE", "1700000000 +0100"),
    ("GIT_COMMITTER_NAME", "Wiki"),
    ("GIT_COMMITTER_EMAIL", "wiki@example.com"),
    ("GIT_COMMITTER_DATE", "1700000000 +0100"),
];

/// Looks up `name` in [`IDENTITY`]. Pass it to `wik::author_and_committer`
/// so wik uses the same identity as Git in these tests.
pub fn identity_var(name: &str) -> Option<String> {
    for (variable, value) in IDENTITY {
        if variable == name {
            return Some(value.to_string());
        }
    }
    None
}
