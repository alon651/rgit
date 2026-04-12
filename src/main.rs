use clap::{Parser, Subcommand};
use pager::Pager;
use std::path::PathBuf;

mod commands;
mod structures;
mod utils;

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
        tree: String,
        ///The id of a parent commit object
        #[arg(short = 'p')]
        parent: Option<String>,
        ///A paragraph in the commit log message
        #[arg(short = 'm')]
        message: Option<String>,
    },
    /// Show commit logs
    Log {
        /// The commit to start from (defaults to HEAD)
        commit: Option<String>,
    },
    /// Unpacks a commit - like checkout
    RestoreCommit {
        /// The commit to unpack
        commit: String,
        /// Where to unpack the commit
        path: PathBuf,
    },
    ///Shows alist of all the refs
    ShowRef {},
    ///Create/List tags
    Tag {
        /// The commit hash to point to if not provided iwll list all tags
        name: Option<String>,
        /// The object to point to - can be a commit or tag if not provided will point to the current head
        // #[arg(value_parser = parse_sha1, requires = "name")]
        #[arg(requires = "name")]
        object: Option<String>,
        /// If enabled create a full tag object
        #[arg(short = 'a', requires = "name")]
        annonate: bool,
    },
    ///Create a branch or list all branches if name wasnt provided
    Branch {
        ///The branch to create, list all branches if not provided
        name: Option<String>,
    },
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
        Commands::Tag {
            name,
            object,
            annonate,
        } => commands::tag::exec(name, object, annonate),
        Commands::Branch { name } => commands::branch::exec(name),
    }
}
