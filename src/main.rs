//! CLI for wik. All logic lives in the lib.rs

use std::env;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use wik::{
    Commit, CommitOutcome, Database, Error, HashAlgorithm, Head, InitOutcome, Object, ObjectId,
    Sha1Crate, Sha1Own, author_and_committer,
};

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

/// Which implementation of the has function to use. The ids are the same
/// the choice only affects how they are computed
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
        /// Also write the objects into repository object database
        #[arg(short)]
        write: bool,

        /// Files to hash
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Create a commit object from a tree and print its id
    CommitTree {
        /// The id of the tree to commit
        tree: ObjectId,

        /// The id of a parent commit (can be given more than once)
        #[arg(short = 'p')]
        parents: Vec<ObjectId>,

        /// The commit message. Without it the message is read from stdin
        #[arg(short = 'm')]
        message: Option<String>,
    },
    /// Record all files of the working folder as a new commit
    Commit {
        /// The commit message
        #[arg(short = 'm')]
        message: String,
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
    let hasher: Box<dyn HashAlgorithm> = match cli.hash_impl {
        HashImpl::Crate => Box::new(Sha1Crate),
        HashImpl::Own => Box::new(Sha1Own),
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
        Command::HashObject { write, paths } => {
            if write {
                let git_dir = wik::find_git_dir(&env::current_dir()?)?;
                let database = Database::new(&git_dir, hasher);
                for path in &paths {
                    let id = database.store(&Object::blob_from_file(path)?)?;
                    println!("{id}");
                }
            } else {
                for path in &paths {
                    let id = wik::hash_file(path, hasher.as_ref())?;
                    println!("{id}");
                }
            }
        }
        Command::CommitTree {
            tree,
            parents,
            message,
        } => {
            let git_dir = wik::find_git_dir(&env::current_dir()?)?;
            let database = Database::new(&git_dir, hasher);

            // Like Git `-m` gets a newline at the end and stdin is used as it is
            let message = match message {
                Some(text) => format!("{text}\n"),
                None => {
                    let mut text = String::new();
                    io::stdin().read_to_string(&mut text)?;
                    text
                }
            };

            let (author, committer) =
                wik::author_and_committer(|name| env::var(name).ok(), wik::now())?;
            let commit = Commit {
                tree,
                parents,
                author,
                committer,
                message,
            };
            println!("{}", database.store_commit(commit)?);
        }
        Command::Commit { message } => {
            let git_dir = wik::find_git_dir(&env::current_dir()?)?;

            // in practice else never runs
            let Some(root) = git_dir.parent() else {
                return Err(Error::NotARepository);
            };

            let database = Database::new(&git_dir, hasher);

            let (author, committer) = author_and_committer(|name| env::var(name).ok(), wik::now())?;

            let outcome = wik::commit(
                root,
                &git_dir,
                &database,
                author,
                committer,
                format!("{message}\n"),
            )?;

            println!("{}", summary(&outcome, &message));
        }
    }
    Ok(())
}

/// Line Git prints after commit like
/// `[main (root-commit) 9du67sa] first commit`
fn summary(outcome: &CommitOutcome, message: &str) -> String {
    let place = match &outcome.head {
        Head::Branch(name) => name.strip_prefix("refs/heads/").unwrap_or(name),
        Head::Detached => "detached HEAD",
    };
    let root = if outcome.root { " (root-commit)" } else { "" };
    let id = outcome.id.to_string();
    let short_id = &id[..7];

    let mut subject = Vec::new();
    for line in message.lines() {
        if line.trim().is_empty() {
            if subject.is_empty() {
                continue;
            }
            break;
        }
        subject.push(line);
    }

    format!("[{place}{root} {short_id}] {}", subject.join(" "))
}
