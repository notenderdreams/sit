use super::common::git_command;
use super::types::Branch;
use crate::error::Result;

/// Returns the name of the currently checked-out branch.
pub fn current_branch() -> Result<String> {
    let output = git_command()
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err("Not a git repository or no commits yet".into());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if branch == "HEAD" {
        return Err("HEAD is detached – cannot push".into());
    }

    Ok(branch)
}

/// Returns (remote, remote_branch) of the configured upstream for the current branch,
/// or `None` when no upstream is set.
pub fn upstream() -> Option<(String, String)> {
    let out = git_command()
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let (remote, branch) = value.split_once('/')?;
    Some((remote.to_owned(), branch.to_owned()))
}

pub fn list_local_branches() -> Result<Vec<Branch>> {
    let output = git_command()
        .args(["branch", "--format=%(refname:short)"])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Could not list branches: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    let current = current_branch().ok();
    let mut branches: Vec<Branch> = String::from_utf8_lossy(&output.stdout)
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
    let output = git_command().args(["switch", name]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to switch branch: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

pub fn create_and_switch_branch(name: &str) -> Result<()> {
    let output = git_command().args(["switch", "-c", name]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to create branch: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}
