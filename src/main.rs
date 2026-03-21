use clap::{Parser, Subcommand};
use pager::Pager;
use parsers::parse_sha1;
use std::path::PathBuf;

mod commands;
mod parsers;
mod structures;

/// a git clone written in rust
#[derive(Parser)]
#[command(name = "rvcs")]
#[command(version,about,long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///Initialize the repo
    Init {
        ///Data dir location
        path: Option<PathBuf>,
    },
    ///Provide content or type and size information for repository objects
    CatFile {
        ///The hash of the object to read
        hash: String,
        /// what action to do
        #[command(flatten)]
        mode: commands::cat_file::CatFileActions,
    },
    ///Compute object ID and optionally creates a blob from a file
    HashObject {
        ///Object to hash
        path: PathBuf,
        /// Actually write the object into the object database
        #[arg(short = 'w')]
        write: bool,
    },
    ///List the contents of a tree object
    LsTree {
        ///Id of a tree
        hash: String,
    },
    ///Create a tree object from the current work tree - later we will enable index and ignoring
    WriteTree {},
    ///Create a new commit object
    CommitTree {
        ///An existing tree object.
        #[arg(value_parser = parse_sha1)]
        tree: String,
        ///The id of a parent commit object
        #[arg(value_parser = parse_sha1, short='p')]
        parent: Option<String>,
        ///A paragraph in the commit log message
        #[arg(short = 'm')]
        message: Option<String>,
    },
    /// Show commit logs
    Log {
        /// The commit to start from (defaults to HEAD)
        #[arg(value_parser = parse_sha1)]
        commit: Option<String>,
    },
    /// Unpacks a commit - like checkout
    RestoreCommit {
        /// The commit to unpack
        #[arg(value_parser = parse_sha1)]
        commit: String,
        /// Where to unpack the commit
        path: PathBuf,
    },
    ShowRef {},
}

fn main() -> anyhow::Result<()> {
    colored::control::set_override(true);

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => commands::init::exec(path),
        Commands::CatFile { hash, mode } => commands::cat_file::exec(&hash, mode.to_mode()),
        Commands::HashObject { path, write } => commands::hash_object::exec(path, write),
        Commands::LsTree { hash } => commands::ls_tree::exec(&hash),
        Commands::WriteTree {} => commands::write_tree::exec(),
        Commands::CommitTree {
            tree,
            parent,
            message,
        } => commands::commit_tree::exec(tree, parent, message),
        Commands::Log { commit } => {
            Pager::with_pager("less -FXR").setup();
            commands::log::exec(commit)
        }
        Commands::RestoreCommit { commit, path } => commands::restore_commit::exec(commit, &path),
        Commands::ShowRef {} => commands::show_ref::exec(),
    }
}
