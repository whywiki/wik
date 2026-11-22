//! Helpers for the library integration tests

use std::path::Path;
use std::process::Command;

/// Runs real Git with `args` inside `dir`. It asserts that the command
/// succeeded and returns what it printed to stdout.
///
/// It ignores the user and system config and it never searches for a repo
/// above `dir`
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
        .output()
        .expect("failed to start git. Is it installed?");

    assert!(
        output.status.success(),
        "git {args:?} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("git printed invalid UTF-8")
}
