use std::process::Command;

use colored::{ColoredString, Colorize};

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
}

impl FileStatus {
    pub fn color_code(&self) -> &str {
        match self {
            Self::Added => "\x1b[32m",
            Self::Modified => "\x1b[33m",
            Self::Deleted => "\x1b[31m",
            Self::Renamed => "\x1b[36m",
            Self::Untracked => "\x1b[90m",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Added => "+",
            Self::Modified => "~",
            Self::Deleted => "✕",
            Self::Renamed => "→",
            Self::Untracked => "?",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            Self::Modified => 0,
            Self::Added => 1,
            Self::Deleted => 2,
            Self::Renamed => 3,
            Self::Untracked => 4,
        }
    }

    pub fn colorize(&self, text: &str) -> ColoredString {
        match self {
            Self::Added => text.green(),
            Self::Modified => text.yellow(),
            Self::Deleted => text.red(),
            Self::Renamed => text.cyan(),
            Self::Untracked => text.bright_black(),
        }
    }
}

pub fn get_status() -> Result<Vec<FileChange>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

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

pub fn stage_files(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("No files selected".into());
    }

    // Reset index first so only selected files are staged
    let _ = Command::new("git").args(["reset", "HEAD"]).output();

    let mut args = vec!["add", "--"];
    let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(refs);

    let output = Command::new("git").args(&args).output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to stage: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

pub fn commit(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Commit failed: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}

/// Returns the name of the currently checked-out branch.
pub fn current_branch() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
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
    // "origin/main" form
    let out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    // Split on first '/'
    let (remote, branch) = s.split_once('/')?;
    Some((remote.to_owned(), branch.to_owned()))
}

pub struct PushResult {
    pub remote: String,
    pub branch: String,
    pub set_upstream: bool,
}

/// Push the current branch.  If no upstream is configured the function sets it
/// to `origin/<branch>` automatically.
pub fn push() -> Result<PushResult, Box<dyn std::error::Error>> {
    let branch = current_branch()?;

    let (remote, set_upstream) = if let Some((r, _)) = upstream() {
        (r, false)
    } else {
        ("origin".to_owned(), true)
    };

    let mut args = vec!["push"];
    let set_upstream_flag;
    if set_upstream {
        args.push("--set-upstream");
        args.push(&remote);
        set_upstream_flag = branch.clone();
        args.push(&set_upstream_flag);
    }

    let output = Command::new("git").args(&args).output()?;

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
