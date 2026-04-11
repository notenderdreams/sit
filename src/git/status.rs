use crate::git::types::{FileChange, FileStatus};

use super::common::git_command;

pub fn get_status() -> Result<Vec<FileChange>, Box<dyn std::error::Error>> {
    let output = git_command().args(["status", "--porcelain"]).output()?;

    if !output.status.success() {
        return Err("Not a git repository".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut changes = Vec::new();

    for line in stdout.lines() {
        if line.len() < 3 {
            continue;
        }

        let index = line.as_bytes()[0];
        let worktree = line.as_bytes()[1];
        let path = line[3..].to_string();

        let status = match (index, worktree) {
            (b'?', b'?') => FileStatus::Untracked,
            (b'A', _) | (_, b'A') => FileStatus::Added,
            (b'D', _) | (_, b'D') => FileStatus::Deleted,
            (b'R', _) => FileStatus::Renamed,
            _ => FileStatus::Modified,
        };

        changes.push(FileChange { path, status });
    }

    Ok(changes)
}
