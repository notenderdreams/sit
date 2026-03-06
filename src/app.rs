use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::categories::CATEGORIES;
use crate::{git, picker, ui};

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

    match cli.command {
        None | Some(Commands::Commit) => interactive_commit(),
        Some(Commands::Categories) => show_categories(),
    }
}

fn interactive_commit() -> Result<(), Box<dyn std::error::Error>> {
    let changes = git::get_status()?;

    if changes.is_empty() {
        println!("\n  {}\n", "No changes to commit".bright_black());
        return Ok(());
    }

    let selected_files = picker::pick_files(changes)?;

    if selected_files.is_empty() {
        println!("  {}\n", "No files selected".bright_black());
        return Ok(());
    }

    let category = ui::select_category()?;
    let message = ui::prompt_message(category)?;
    let description = ui::prompt_description()?;

    let full_message = if category == "none" {
        if description.trim().is_empty() {
            message.clone()
        } else {
            format!("{}\n\n{}", message, description)
        }
    } else if description.trim().is_empty() {
        format!("{}: {}", category, message)
    } else {
        format!("{}: {}\n\n{}", category, message, description)
    };

    git::stage_files(&selected_files)?;
    git::commit(&full_message)?;
    ui::print_success(&full_message);

    Ok(())
}

fn show_categories() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n  {}\n", "Commit Categories:".bold());
    for cat in CATEGORIES {
        println!(
            "    {}  {}  {}",
            cat.emoji,
            cat.name.bold(),
            cat.description.bright_black()
        );
    }
    println!();
    Ok(())
}
