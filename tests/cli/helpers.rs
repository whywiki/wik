//! Helpers for the end-to-end tests

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// The author and committer used by every test
pub const IDENTITY: [(&str, &str); 6] = [
    ("GIT_AUTHOR_NAME", "Wiki"),
    ("GIT_AUTHOR_EMAIL", "wiki@example.com"),
    ("GIT_AUTHOR_DATE", "1700000000 +0100"),
    ("GIT_COMMITTER_NAME", "Wiki"),
    ("GIT_COMMITTER_EMAIL", "wiki@example.com"),
    ("GIT_COMMITTER_DATE", "1700000000 +0100"),
];

/// Runs `wik` binary (Cargo builds it) with `args` inside `dir`,
/// with [`IDENTITY`] as author and committer.
///
/// Use it when a test needs to change something before running, for
/// example remove a variable. Otherwise use [`wik`].
pub fn wik_command(dir: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wik"));
    command.args(args).current_dir(dir).envs(IDENTITY);
    command
}

/// Runs `wik` binary with `args` inside `dir`
pub fn wik(dir: &Path, args: &[&str]) -> Output {
    wik_command(dir, args)
        .output()
        .expect("failed to start wik")
}

/// Runs `wik` binary with `args` inside `dir` and gives it `input` on
/// stdin
pub fn wik_with_input(dir: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = wik_command(dir, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start wik");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input)
        .expect("failed to write to wik");
    child.wait_with_output().expect("failed to wait for wik")
}

/// Command stdout output to text
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Command stderr output to text
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
