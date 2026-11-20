use super::common::{git_output, git_run};
use super::types::Branch;
use crate::error::Result;

/// Returns the name of the currently checked-out branch.
pub fn current_branch() -> Result<String> {
    let branch = git_run(
        &["rev-parse", "--abbrev-ref", "HEAD"],
        "Not a git repository or no commits yet",
    )?;
    if branch == "HEAD" {
        return Err("HEAD is detached – cannot push".into());
    }
    Ok(branch)
}

/// Returns (remote, remote_branch) of the configured upstream for the current branch,
/// or `None` when no upstream is set.
pub fn upstream() -> Option<(String, String)> {
    let out = git_output(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"]).ok()?;

    if !out.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let (remote, branch) = value.split_once('/')?;
    Some((remote.to_owned(), branch.to_owned()))
}

pub fn list_local_branches() -> Result<Vec<Branch>> {
    let listing = git_run(
        &["branch", "--format=%(refname:short)"],
        "Could not list branches",
    )?;

    let current = current_branch().ok();
    let mut branches: Vec<Branch> = listing
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| Branch {
            name: name.to_owned(),
            is_current: current.as_deref() == Some(name),
        })
        .collect();

    branches.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(branches)
}

pub fn switch_branch(name: &str) -> Result<()> {
    git_run(&["switch", name], "Failed to switch branch").map(drop)
}

pub fn create_and_switch_branch(name: &str) -> Result<()> {
    git_run(&["switch", "-c", name], "Failed to create branch").map(drop)
}
