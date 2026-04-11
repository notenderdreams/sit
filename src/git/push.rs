use super::branches::{current_branch, upstream};
use super::common::git_command;
use super::types::PushResult;

/// Push the current branch. If no upstream is configured, sets it to `origin/<branch>`.
pub fn push() -> Result<PushResult, Box<dyn std::error::Error>> {
    let branch = current_branch()?;

    let (remote, set_upstream) = if let Some((remote_name, _)) = upstream() {
        (remote_name, false)
    } else {
        ("origin".to_owned(), true)
    };

    let mut args = vec!["push"];
    let set_upstream_branch;
    if set_upstream {
        args.push("--set-upstream");
        args.push(&remote);
        set_upstream_branch = branch.clone();
        args.push(&set_upstream_branch);
    }

    let output = git_command().args(&args).output()?;

    if output.status.success() {
        Ok(PushResult {
            remote,
            branch,
            set_upstream,
        })
    } else {
        Err(format!(
            "Push failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

/// Push the current branch with `--force-with-lease`.
/// If no upstream is configured, sets it to `origin/<branch>`.
pub fn push_force() -> Result<PushResult, Box<dyn std::error::Error>> {
    let branch = current_branch()?;

    let (remote, set_upstream) = if let Some((remote_name, _)) = upstream() {
        (remote_name, false)
    } else {
        ("origin".to_owned(), true)
    };

    let mut args = vec!["push", "--force-with-lease"];
    let set_upstream_branch;
    if set_upstream {
        args.push("--set-upstream");
        args.push(&remote);
        set_upstream_branch = branch.clone();
        args.push(&set_upstream_branch);
    }

    let output = git_command().args(&args).output()?;

    if output.status.success() {
        Ok(PushResult {
            remote,
            branch,
            set_upstream,
        })
    } else {
        Err(format!(
            "Force push failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}
