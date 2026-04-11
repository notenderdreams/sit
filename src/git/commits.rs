use super::common::git_command;

pub fn commit(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = git_command().args(["commit", "-m", message]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Commit failed: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}

/// Amend the last commit, optionally changing the message.
pub fn commit_amend(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = git_command()
        .args(["commit", "--amend", "-m", message])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Amend failed: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}

/// Return the subject line (first line) of the last commit message.
pub fn last_commit_message() -> Result<String, Box<dyn std::error::Error>> {
    let output = git_command().args(["log", "-1", "--format=%B"]).output()?;

    if !output.status.success() {
        return Err("No commits found".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Return the list of files changed in HEAD.
pub fn last_commit_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = git_command()
        .args(["diff-tree", "--no-commit-id", "-r", "--name-only", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err("Could not list files in last commit".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_owned())
        .collect())
}

/// Undo the last commit with a soft reset (changes stay staged).
pub fn soft_reset() -> Result<(), Box<dyn std::error::Error>> {
    let output = git_command().args(["reset", "--soft", "HEAD~1"]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Reset failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}
