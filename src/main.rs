//! CLI for wik. All logic lives in the lib.rs

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use wik::{HashAlgorithm, InitOutcome, Sha1Crate, Sha1Own};

/// wik: git compatible Version Control System
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Which implementation computes hashes
    #[arg(long, global = true, value_enum, default_value_t = HashImpl::Crate)]
    hash_impl: HashImpl,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum HashImpl {
    /// The sha1 crate from the RustCrypto project
    Crate,
    /// wik own implementation
    Own,
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
    let hasher: &dyn HashAlgorithm = match cli.hash_impl {
        HashImpl::Crate => &Sha1Crate,
        HashImpl::Own => &Sha1Own,
    };

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
                let id = wik::hash_file(path, hasher)?;
                println!("{id}");
            }
        }
    }
    Ok(())
}
