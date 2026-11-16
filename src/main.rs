//! CLI for wik. All logic lives in the lib.rs

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use wik::{InitOutcome, Sha1Crate};

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
    /// Calculate the object id of files as blobs
    HashObject {
        /// Files to hash
        #[arg(required = true)]
        paths: Vec<PathBuf>,
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
        Command::Init { path } => match wik::init(&path)? {
            InitOutcome::Created(git_dir) => {
                println!("Initialized empty Wik repository in {}/", git_dir.display());
            }
            InitOutcome::Reinitialized(git_dir) => {
                println!(
                    "Reinitialized existing Wik repository in {}/",
                    git_dir.display()
                );
            }
        },
        Command::HashObject { paths } => {
            for path in &paths {
                let id = wik::hash_file(path, &Sha1Crate)?;
                println!("{id}");
            }
        }
    }
    Ok(())
}
