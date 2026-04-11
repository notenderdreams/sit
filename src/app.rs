use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::config::Config;
use crate::{git, hooks, picker, print, ui};

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

    /// Create .sit/config.toml and hook templates in the current directory
    Init,

    /// Push the current branch to its upstream (sets upstream if unset)
    #[command(alias = "p")]
    Push,

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

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg = Config::load();

    match cli.command {
        None | Some(Commands::Commit) => interactive_commit(&cfg),
        Some(Commands::Categories) => show_categories(&cfg),
        Some(Commands::Branch) => switch_branch(),
        Some(Commands::Init) => init_config(),
        Some(Commands::Push) => push_branch(),
        Some(Commands::Amend) => amend_commit(&cfg),
        Some(Commands::Undo) => undo_commit(),
        Some(Commands::Connect { username, repo }) => connect_repo(&username, &repo),
        Some(Commands::CategoryShortcut(args)) => category_shortcut_commit(&cfg, &args),
    }
}

fn category_shortcut_commit(
    cfg: &Config,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(category) = args.first().map(|s| s.as_str()) else {
        return interactive_commit(cfg);
    };

    if !cfg.categories.iter().any(|c| c.name == category) {
        return Err(format!("Unknown command or category: {category}").into());
    }

    let inline_message = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    commit_with_category(cfg, category, inline_message)
}

fn commit_with_category(
    cfg: &Config,
    category: &str,
    inline_message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    finalize_commit_with_files(cfg, category, inline_message, selected_files)
}

fn finalize_commit_with_files(
    cfg: &Config,
    category: &str,
    inline_message: Option<String>,
    selected_files: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let module = if cfg.has_modules() {
        ui::select_module(&cfg.modules)?
    } else {
        None
    };

    let message = match inline_message {
        Some(m) if !m.trim().is_empty() => m,
        _ => ui::prompt_message(category)?,
    };

    let description = if cfg.commit.ask_description {
        ui::prompt_description()?
    } else {
        String::new()
    };

    let full_message = cfg.format_commit(category, module, &message, &description);

    let subject = full_message.lines().next().unwrap_or(&full_message);

    if !ui::confirm_commit(subject, "", &selected_files)? {
        print::blank();
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    hooks::run_hook(
        "pre-commit",
        hooks::HookKind::Pre,
        &[
            ("SIT_CATEGORY", category),
            ("SIT_MESSAGE", &full_message),
            ("SIT_FILES", &selected_files.join(":")),
        ],
    )?;

    git::stage_files(&selected_files)?;
    git::commit(&full_message)?;

    print::blank();
    print::success_with_details("Committed", &full_message);
    print::blank();

    hooks::run_hook(
        "post-commit",
        hooks::HookKind::Post,
        &[("SIT_CATEGORY", category), ("SIT_MESSAGE", &full_message)],
    )?;

    if cfg.commit.auto_push {
        do_push()?;
    } else if ui::confirm_push()?
        && let Err(e) = do_push()
    {
        print::error(&e.to_string());
        print::blank();
    }

    Ok(())
}

fn switch_branch() -> Result<(), Box<dyn std::error::Error>> {
    let branches = git::list_local_branches()?;
    let selected = ui::select_branch(&branches)?;

    if branches.iter().any(|b| b.name == selected) {
        git::switch_branch(&selected)?;
        print::blank();
        print::success_with_details("Switched", &format!("→ {selected}"));
        print::blank();
        return Ok(());
    }

    if !ui::confirm_create_branch(&selected)? {
        print::blank();
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    git::create_and_switch_branch(&selected)?;
    print::blank();
    print::success_with_details("Created and switched", &format!("→ {selected}"));
    print::blank();
    Ok(())
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

    let category = ui::select_category(&cfg.categories, cfg.commit.attach_emoji)?;

    finalize_commit_with_files(cfg, category, None, selected_files)
}

fn show_categories(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    print::blank();
    print::header("Commit Categories:");
    print::blank();
    for cat in &cfg.categories {
        if cfg.commit.attach_emoji {
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

fn push_branch() -> Result<(), Box<dyn std::error::Error>> {
    print::blank();
    do_push()
}

fn do_push() -> Result<(), Box<dyn std::error::Error>> {
    hooks::run_hook("pre-push", hooks::HookKind::Pre, &[])?;
    let result = git::push()?;
    if result.set_upstream {
        print::success_with_details(
            "Pushed",
            &format!("→ {}/{} (upstream set)", result.remote, result.branch),
        );
    } else {
        print::success_with_details("Pushed", &format!("→ {}/{}", result.remote, result.branch));
    }
    hooks::run_hook(
        "post-push",
        hooks::HookKind::Post,
        &[
            ("SIT_REMOTE", &result.remote),
            ("SIT_BRANCH", &result.branch),
        ],
    )?;
    print::blank();
    Ok(())
}

fn amend_commit(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // ── Fetch the last commit's message and file list ────────────────────────
    let last_message = git::last_commit_message()?;
    let last_files = git::last_commit_files()?;

    // ── Optionally stage additional working-tree changes ─────────────────────
    let wip = git::get_status()?;
    let staged_files: Vec<String> = if !wip.is_empty() {
        print::blank();
        print::hint("Select additional files to include in the amended commit (Esc to skip):");
        picker::pick_files(wip).unwrap_or_default()
    } else {
        vec![]
    };

    if !staged_files.is_empty() {
        git::stage_files(&staged_files)?;
    }

    // ── Build the file list shown in the preview ──────────────────────────────
    let mut preview_files = last_files;
    for f in &staged_files {
        if !preview_files.contains(f) {
            preview_files.push(f.clone());
        }
    }

    // ── Let the user edit the commit message (pre-filled) ────────────────────
    let new_message = ui::prompt_amend_message(&last_message)?;

    // ── Preview & confirm ────────────────────────────────────────────────────
    let subject = new_message.lines().next().unwrap_or(&new_message);

    if !ui::confirm_commit(subject, "", &preview_files)? {
        if !staged_files.is_empty() {
            let _ = git::unstage_files(&staged_files);
        }
        print::blank();
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    // ── Amend ────────────────────────────────────────────────────────────────
    git::commit_amend(&new_message)?;

    print::blank();
    print::success_with_details("Amended", &new_message);
    print::blank();

    // ── Push with --force-with-lease (history was rewritten) ─────────────────
    if cfg.commit.auto_push {
        do_push_force()?;
    } else if ui::confirm_push()?
        && let Err(e) = do_push_force()
    {
        print::error(&e.to_string());
        print::blank();
    }

    Ok(())
}

fn undo_commit() -> Result<(), Box<dyn std::error::Error>> {
    let last_message = git::last_commit_message()?;
    let last_files = git::last_commit_files()?;

    // Show what will be undone
    print::blank();
    print::header("Undo last commit?");
    print::blank();
    println!(
        "    {}",
        last_message.lines().next().unwrap_or(&last_message).bold()
    );
    print::blank();
    print::hint("Files (will remain staged):");
    let last = last_files.len().saturating_sub(1);
    for (i, f) in last_files.iter().enumerate() {
        let branch = if i == last { "└──" } else { "├──" };
        println!("    \x1b[2m{branch}\x1b[0m {f}");
    }
    print::blank();

    if !ui::confirm_undo()? {
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    git::soft_reset()?;

    print::blank();
    print::success("Undone — changes are staged and ready to re-commit");
    print::blank();

    Ok(())
}

fn do_push_force() -> Result<(), Box<dyn std::error::Error>> {
    hooks::run_hook("pre-push", hooks::HookKind::Pre, &[("SIT_FORCE", "1")])?;
    let result = git::push_force()?;
    if result.set_upstream {
        print::success_with_details(
            "Pushed",
            &format!("→ {}/{} (upstream set)", result.remote, result.branch),
        );
    } else {
        print::success_with_details(
            "Pushed",
            &format!("→ {}/{} (force-with-lease)", result.remote, result.branch),
        );
    }
    hooks::run_hook(
        "post-push",
        hooks::HookKind::Post,
        &[
            ("SIT_REMOTE", &result.remote),
            ("SIT_BRANCH", &result.branch),
            ("SIT_FORCE", "1"),
        ],
    )?;
    print::blank();
    Ok(())
}

fn connect_repo(username: &str, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    print::blank();
    print::header(&format!("Connecting to github.com/{username}/{repo}"));
    print::blank();

    git::remote_add_origin(username, repo)?;
    print::success_with_details(
        "Remote added",
        &format!("origin → https://github.com/{username}/{repo}.git"),
    );

    git::branch_rename_to_main()?;
    print::success_with_details("Branch", "renamed to main");

    git::push_origin_main()?;
    print::success_with_details("Pushed", "→ origin/main (upstream set)");

    print::blank();
    Ok(())
}

fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    let sit_dir = Path::new(".sit");
    let hooks_dir = sit_dir.join("hooks");
    let config_path = sit_dir.join("config.toml");
    let legacy_path = Path::new(".sitrc");

    if config_path.exists() {
        return Err(".sit/config.toml already exists in the current directory".into());
    }

    std::fs::create_dir_all(&hooks_dir)?;

    let migrated_legacy = if legacy_path.exists() {
        match std::fs::rename(legacy_path, &config_path) {
            Ok(()) => true,
            Err(_) => {
                // Fallback for cross-device filesystems.
                std::fs::copy(legacy_path, &config_path)?;
                std::fs::remove_file(legacy_path)?;
                true
            }
        }
    } else {
        std::fs::write(&config_path, Config::default_toml())?;
        false
    };

    write_hook_template(&hooks_dir.join("pre-commit"), hooks::PRE_COMMIT_TEMPLATE)?;
    write_hook_template(&hooks_dir.join("post-commit"), hooks::POST_COMMIT_TEMPLATE)?;
    write_hook_template(&hooks_dir.join("pre-push"), hooks::PRE_PUSH_TEMPLATE)?;
    write_hook_template(&hooks_dir.join("post-push"), hooks::POST_PUSH_TEMPLATE)?;

    print::blank();
    if migrated_legacy {
        print::success("Migrated .sitrc to .sit/config.toml and created hook templates");
    } else {
        print::success("Created .sit/config.toml and hook templates");
    }
    print::hint("Edit .sit/config.toml and uncomment hook examples in .sit/hooks as needed.");
    print::blank();

    Ok(())
}

fn write_hook_template(
    path: &std::path::Path,
    template: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        std::fs::write(path, template)?;
    }
    Ok(())
}
