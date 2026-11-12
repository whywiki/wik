//! Helpers for the integration tests

use std::path::Path;
use std::process::Command;

/// Runs real Git with `args` inside `dir`. It asserts that the command
/// succeeded and returns what it printed to stdout.
///
/// It ignore the user and system config and it never seraches for a repo
/// above `dir`
pub fn git(dir: &Path, args: &[&str]) -> String {
    let ceiling = dir.parent().unwrap_or(dir);

    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        // git ignores user cdonfigs
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
