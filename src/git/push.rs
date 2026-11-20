use super::branches::{current_branch, upstream};
use super::common::git_run;
use super::types::PushResult;
use crate::error::Result;

/// Push the current branch. If no upstream is configured, sets it to `origin/<branch>`.
pub fn push() -> Result<PushResult> {
    push_inner(false)
}

/// Push the current branch with `--force-with-lease`.
pub fn push_force() -> Result<PushResult> {
    push_inner(true)
}

fn push_inner(force: bool) -> Result<PushResult> {
    let branch = current_branch()?;
    let (remote, set_upstream) = match upstream() {
        Some((remote, _)) => (remote, false),
        None => ("origin".to_owned(), true),
    };

    let mut args = vec!["push"];
    if force {
        args.push("--force-with-lease");
    }
    if set_upstream {
        args.extend(["--set-upstream", &remote, &branch]);
    }

    git_run(
        &args,
        if force {
            "Force push failed"
        } else {
            "Push failed"
        },
    )?;
    Ok(PushResult {
        remote,
        branch,
        set_upstream,
    })
}
