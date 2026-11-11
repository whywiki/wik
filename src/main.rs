//! CLI for wik. All logis lives in the lib.rs

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// wik: git compatible Version Control System
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Creates an empty Wik repository
    Init {
        /// Directory to create the repository in
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("fatal: {err}");
            ExitCode::from(128)
        }
    }
}

/// Runs commands
fn run(cli: Cli) -> wik::Result<()> {
    match cli.command {
        Command::Init { path } => {
            let git_dir = wik::init(&path)?;
            println!("Initialized empty Wik repository in {}/", git_dir.display());
        }
    }
    Ok(())
}
