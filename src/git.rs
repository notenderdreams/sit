use std::process::Command;

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
}

pub fn get_repo_root() -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(std::path::PathBuf::from(path))
    } else {
        None
    }
}

fn git_command() -> Command {
    let mut cmd = Command::new("git");
    if let Some(root) = get_repo_root() {
        cmd.current_dir(root);
    }
    cmd
}

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

pub fn stage_files(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("No files selected".into());
    }

    // Reset index first so only selected files are staged
    let _ = git_command().args(["reset", "HEAD"]).output();

    let mut args = vec!["add", "--"];
    let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(refs);

    let output = git_command().args(&args).output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to stage: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

pub fn unstage_files(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Ok(());
    }

    let mut args = vec!["restore", "--staged", "--"];
    let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(refs);

    let output = git_command().args(&args).output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to unstage: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(())
}

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
        .map(|l| l.to_owned())
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

/// Returns the name of the currently checked-out branch.
pub fn current_branch() -> Result<String, Box<dyn std::error::Error>> {
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
    // "origin/main" form
    let out = git_command()
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

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
}

pub fn list_local_branches() -> Result<Vec<Branch>, Box<dyn std::error::Error>> {
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

pub fn switch_branch(name: &str) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn create_and_switch_branch(name: &str) -> Result<(), Box<dyn std::error::Error>> {
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

/// Push the current branch with `--force-with-lease` (safe force for amends).
/// If no upstream is configured, sets it to `origin/<branch>` automatically.
pub fn push_force() -> Result<PushResult, Box<dyn std::error::Error>> {
    let branch = current_branch()?;

    let (remote, set_upstream) = if let Some((r, _)) = upstream() {
        (r, false)
    } else {
        ("origin".to_owned(), true)
    };

    let mut args = vec!["push", "--force-with-lease"];
    let set_upstream_flag;
    if set_upstream {
        args.push("--set-upstream");
        args.push(&remote);
        set_upstream_flag = branch.clone();
        args.push(&set_upstream_flag);
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

/// Add a GitHub remote as `origin`.
pub fn remote_add_origin(username: &str, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
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
pub fn branch_rename_to_main() -> Result<(), Box<dyn std::error::Error>> {
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
pub fn push_origin_main() -> Result<(), Box<dyn std::error::Error>> {
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

fn is_release_tag(tag: &str) -> bool {
    let trimmed = tag.trim();
    let version = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let mut parts = version.split('.');

    let (Some(major), Some(minor), Some(patch), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };

    !major.is_empty()
        && !minor.is_empty()
        && !patch.is_empty()
        && major.chars().all(|c| c.is_ascii_digit())
        && minor.chars().all(|c| c.is_ascii_digit())
        && patch.chars().all(|c| c.is_ascii_digit())
}

/// Return the latest semantic release tag (e.g. v1.2.3) if present.
pub fn latest_release_tag() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let output = git_command()
        .args(["tag", "--list", "--sort=-v:refname"])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to list tags: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    let latest = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|tag| is_release_tag(tag))
        .map(str::to_owned);

    Ok(latest)
}

/// Create a local release tag.
pub fn create_tag(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !is_release_tag(tag) {
        return Err("Tag must be semver-like (v1.2.3 or 1.2.3)".into());
    }

    let output = git_command().args(["tag", tag]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to create tag: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

/// Push a tag to the default remote.
pub fn push_tag(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let remote = if let Some((r, _)) = upstream() {
        r
    } else {
        "origin".to_owned()
    };

    let output = git_command().args(["push", &remote, tag]).output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to push tag: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}
