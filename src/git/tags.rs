use super::branches::upstream;
use super::common::git_command;
use crate::error::Result;

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
pub fn latest_release_tag() -> Result<Option<String>> {
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
pub fn create_tag(tag: &str) -> Result<()> {
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
pub fn push_tag(tag: &str) -> Result<()> {
    let remote = if let Some((remote_name, _)) = upstream() {
        remote_name
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
