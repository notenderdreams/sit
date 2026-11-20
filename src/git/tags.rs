use super::branches::upstream;
use super::common::git_run;
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
    let out = git_run(
        &["tag", "--list", "--sort=-v:refname"],
        "Failed to list tags",
    )?;
    Ok(out
        .lines()
        .map(str::trim)
        .find(|tag| is_release_tag(tag))
        .map(str::to_owned))
}

/// Create a local release tag.
pub fn create_tag(tag: &str) -> Result<()> {
    if !is_release_tag(tag) {
        return Err("Tag must be semver-like (v1.2.3 or 1.2.3)".into());
    }
    git_run(&["tag", tag], "Failed to create tag").map(drop)
}

/// Push a tag to the default remote.
pub fn push_tag(tag: &str) -> Result<()> {
    let remote = upstream().map_or_else(|| "origin".to_owned(), |(remote, _)| remote);
    git_run(&["push", &remote, tag], "Failed to push tag").map(drop)
}
