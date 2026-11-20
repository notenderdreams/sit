use super::common::{git_output, git_run};
use crate::error::Result;

/// Add a GitHub remote as `origin`.
pub fn remote_add_origin(username: &str, repo: &str) -> Result<()> {
    let url = format!("https://github.com/{username}/{repo}.git");
    git_run(&["remote", "add", "origin", &url], "Failed to add remote").map(drop)
}

/// Rename the current branch to `main`.
pub fn branch_rename_to_main() -> Result<()> {
    git_run(&["branch", "-M", "main"], "Failed to rename branch").map(drop)
}

/// Push to `origin main` and set it as the upstream.
pub fn push_origin_main() -> Result<()> {
    git_run(&["push", "-u", "origin", "main"], "Push failed").map(drop)
}

/// Returns true when the named remote exists.
pub fn has_remote(name: &str) -> bool {
    git_output(&["remote", "get-url", name]).is_ok_and(|o| o.status.success())
}
