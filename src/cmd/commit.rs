use crate::config::Config;
use crate::error::Result;
use crate::{git, hooks, picker, print, ui};

use super::push::do_push;

pub fn interactive_commit(cfg: &Config) -> Result<()> {
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

pub fn commit_with_category_shortcut(
    cfg: &Config,
    category: &str,
    inline_message: Option<String>,
) -> Result<()> {
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
) -> Result<()> {
    let module = if cfg.has_modules() {
        let default_module = cfg.recommended_module_name(&selected_files);
        ui::select_module(&cfg.modules, default_module)?
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
