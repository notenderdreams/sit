use super::common::git_command;
use crate::error::Result;

/// Add a GitHub remote as `origin`.
pub fn remote_add_origin(username: &str, repo: &str) -> Result<()> {
    let url = format!("https://github.com/{username}/{repo}.git");
    let output = git_command()
        .args(["remote", "add", "origin", &url])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to add remote: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

/// Rename the current branch to `main`.
pub fn branch_rename_to_main() -> Result<()> {
    let output = git_command().args(["branch", "-M", "main"]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to rename branch: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

/// Push to `origin main` and set it as the upstream.
pub fn push_origin_main() -> Result<()> {
    let output = git_command()
        .args(["push", "-u", "origin", "main"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Push failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}
