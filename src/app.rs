use clap::{Parser, Subcommand};

use crate::cmd;
use crate::config::Config;
use crate::error::Result;

/// Structured interactive commits
#[derive(Parser)]
#[command(name = "sit", version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive commit
    #[command(alias = "c")]
    Commit,

    /// List available commit categories
    #[command(alias = "cat")]
    Categories,

    /// Switch branches with a searchable picker
    #[command(alias = "b")]
    Branch,

    /// Show a graph view of recent commits
    #[command(alias = "l")]
    Log,

    /// Create .sit/config.toml and hook templates in the current directory
    Init,

    /// Push the current branch to its upstream (sets upstream if unset)
    #[command(alias = "p")]
    Push,

    /// Create and push a release tag (e.g. v0.2.1)
    #[command(alias = "rel")]
    Release,

    /// Amend the last commit (message and/or staged changes)
    #[command(alias = "a")]
    Amend,

    /// Undo the last commit (soft reset – changes stay staged)
    #[command(alias = "u")]
    Undo,

    /// Connect the repository to a GitHub remote and push to main
    #[command(alias = "con")]
    Connect {
        /// GitHub username
        #[arg(value_name = "USERNAME")]
        username: String,

        /// Repository name (without .git)
        #[arg(value_name = "REPO")]
        repo: String,
    },

    /// Category shortcut (e.g. `sit feat`, `sit wip`)
    #[command(external_subcommand)]
    CategoryShortcut(Vec<String>),
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load();

    match cli.command {
        None | Some(Commands::Commit) => cmd::commit::interactive_commit(&cfg),
        Some(Commands::Categories) => cmd::categories::show_categories(&cfg),
        Some(Commands::Branch) => cmd::branch::switch_branch(),
        Some(Commands::Log) => cmd::log::show_log(),
        Some(Commands::Init) => cmd::init::init_config(),
        Some(Commands::Push) => cmd::push::push_branch(),
        Some(Commands::Release) => cmd::release::release_tag(),
        Some(Commands::Amend) => cmd::amend::amend_commit(&cfg),
        Some(Commands::Undo) => cmd::undo::undo_commit(),
        Some(Commands::Connect { username, repo }) => cmd::connect::connect_repo(&username, &repo),
        Some(Commands::CategoryShortcut(args)) => {
            cmd::forward::handle_external_or_category(&cfg, &args)
        }
    }
}
