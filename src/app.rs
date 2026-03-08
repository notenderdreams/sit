use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::config::Config;
use crate::{clone, git, picker, print, ui};

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

    /// Clone a GitHub repository
    Clone {
        /// Repository URL (https://github.com/user/repo or user/repo)
        #[arg(value_name = "URL")]
        repo: String,
    },

    /// Create a .sitrc with default config in the current directory
    Init,

    /// Push the current branch to its upstream (sets upstream if unset)
    #[command(alias = "p")]
    Push,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg = Config::load();

    match cli.command {
        None | Some(Commands::Commit) => interactive_commit(&cfg),
        Some(Commands::Categories) => show_categories(&cfg),
        Some(Commands::Clone { repo }) => clone_repo(&cfg, &repo),
        Some(Commands::Init) => init_config(),
        Some(Commands::Push) => push_branch(),
    }
}

fn interactive_commit(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let changes = git::get_status()?;

    if changes.is_empty() {
        print::blank();
        print::hint("No changes to commit");
        print::blank();
        return Ok(());
    }

    let selected_files = picker::pick_files(changes)?;

    if selected_files.is_empty() {
        print::hint("No files selected");
        print::blank();
        return Ok(());
    }

    let category = ui::select_category(&cfg.categories, cfg.commit.show_emoji)?;

    let module = if cfg.has_modules() {
        ui::select_module(&cfg.modules)?
    } else {
        None
    };

    let message = ui::prompt_message(category)?;

    let description = if cfg.commit.ask_description {
        ui::prompt_description()?
    } else {
        String::new()
    };

    let full_message = cfg.format_commit(category, module, &message, &description);

    // ── Preview & confirm ────────────────────────────────────────────────────
    let emoji = cfg
        .categories
        .iter()
        .find(|c| c.name == category)
        .map(|c| c.emoji.as_str())
        .unwrap_or("");

    let subject = full_message.lines().next().unwrap_or(&full_message);

    if !ui::confirm_commit(subject, emoji, &selected_files)? {
        print::blank();
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    // ── Commit ───────────────────────────────────────────────────────────────
    git::stage_files(&selected_files)?;
    git::commit(&full_message)?;

    print::blank();
    print::success_with_details("Committed", &full_message);
    print::blank();

    // ── Push ─────────────────────────────────────────────────────────────────
    if cfg.commit.auto_push {
        do_push()?;
    } else if ui::confirm_push()? {
        if let Err(e) = do_push() {
            print::error(&e.to_string());
            print::blank();
        }
    }

    Ok(())
}

fn show_categories(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    print::blank();
    print::header("Commit Categories:");
    print::blank();
    for cat in &cfg.categories {
        if cfg.commit.show_emoji {
            println!(
                "    {}  {}  {}",
                cat.emoji,
                cat.name.bold(),
                cat.description.bright_black()
            );
        } else {
            println!(
                "    {}  {}",
                cat.name.bold(),
                cat.description.bright_black()
            );
        }
    }
    print::blank();
    Ok(())
}

fn clone_repo(cfg: &Config, repo_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target_path = clone::clone_repo(repo_url, &cfg.clone.dir)?;

    // Output the cloned directory path for shell integration
    // Usage: cd $(sit clone https://github.com/user/repo)
    println!("{}", target_path.display());

    Ok(())
}

fn push_branch() -> Result<(), Box<dyn std::error::Error>> {
    print::blank();
    do_push()
}

fn do_push() -> Result<(), Box<dyn std::error::Error>> {
    let result = git::push()?;
    if result.set_upstream {
        print::success_with_details(
            "Pushed",
            &format!("→ {}/{} (upstream set)", result.remote, result.branch),
        );
    } else {
        print::success_with_details(
            "Pushed",
            &format!("→ {}/{}", result.remote, result.branch),
        );
    }
    print::blank();
    Ok(())
}

fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(".sitrc");

    if path.exists() {
        return Err(".sitrc already exists in the current directory".into());
    }

    std::fs::write(path, Config::default_toml())?;

    print::blank();
    print::success("Created .sitrc with default config");
    print::hint("Edit it to customise categories, modules, and commit settings.");
    print::blank();

    Ok(())
}
