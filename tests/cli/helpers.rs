//! Helpers for the end-to-end tests

use std::path::Path;
use std::process::{Command, Output};

/// Runs `wik` binary (Cargo builds it) with `args` inside `dir`
pub fn wik(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wik"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to start wik")
}

/// Command stdout output to text
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Command stderr output to text
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
