use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::config::Config;
use crate::{git, picker, print, ui};

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
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg = Config::load();

    match cli.command {
        None | Some(Commands::Commit) => interactive_commit(&cfg),
        Some(Commands::Categories) => show_categories(&cfg),
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
    let message = ui::prompt_message(category)?;

    let description = if cfg.commit.ask_description {
        ui::prompt_description()?
    } else {
        String::new()
    };

    let full_message = cfg.format_commit(category, &message, &description);

    git::stage_files(&selected_files)?;
    git::commit(&full_message)?;

    print::blank();
    print::success_with_details("Committed", &full_message);
    print::blank();

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
