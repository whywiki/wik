//! CLI for wik. All logis lives in the lib.rs

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("fatal: {err}");
            ExitCode::from(128)
        }
    }
}

fn run() -> wik::Result<()> {
    Ok(())
}
