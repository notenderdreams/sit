use super::common::git_run;
use crate::error::Result;

pub fn commit(message: &str) -> Result<()> {
    git_run(&["commit", "-m", message], "Commit failed").map(drop)
}

/// Amend the last commit, optionally changing the message.
pub fn commit_amend(message: &str) -> Result<()> {
    git_run(&["commit", "--amend", "-m", message], "Amend failed").map(drop)
}

/// Return the full message of the last commit.
pub fn last_commit_message() -> Result<String> {
    git_run(&["log", "-1", "--format=%B"], "No commits found")
}

/// Return the list of files changed in HEAD (works for the root commit too).
pub fn last_commit_files() -> Result<Vec<String>> {
    let out = git_run(
        &[
            "diff-tree",
            "--root",
            "--no-commit-id",
            "-r",
            "--name-only",
            "HEAD",
        ],
        "Could not list files in last commit",
    )?;
    Ok(out.lines().map(str::to_owned).collect())
}

/// Undo the last commit with a soft reset (changes stay staged).
pub fn soft_reset() -> Result<()> {
    git_run(&["reset", "--soft", "HEAD~1"], "Reset failed").map(drop)
}
