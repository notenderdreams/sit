use crate::config::Config;
use crate::error::Result;
use crate::{git, picker, print, ui};

use super::push::do_push_force;

pub fn amend_commit(cfg: &Config, hook_env: &[(String, String)]) -> Result<()> {
    let last_message = git::last_commit_message()?;
    let last_files = git::last_commit_files()?;

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

    let mut preview_files = last_files;
    for f in &staged_files {
        if !preview_files.contains(f) {
            preview_files.push(f.clone());
        }
    }

    let new_message = ui::prompt_amend_message(&last_message)?;

    let subject = new_message.lines().next().unwrap_or(&new_message);

    if !ui::confirm_commit(subject, &preview_files)? {
        if !staged_files.is_empty() {
            let _ = git::unstage_files(&staged_files);
        }
        print::blank();
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    git::commit_amend(&new_message)?;

    print::blank();
    print::success_with_details("Amended", &new_message);
    print::blank();

    let can_push = git::upstream().is_some() || git::has_remote("origin");
    if !can_push {
        print::hint("No remote configured; skipping push");
        print::blank();
        return Ok(());
    }

    if cfg.commit.auto_push {
        do_push_force(hook_env)?;
    } else if ui::confirm_push()?
        && let Err(e) = do_push_force(hook_env)
    {
        print::error(&e.to_string());
        print::blank();
    }

    Ok(())
}
