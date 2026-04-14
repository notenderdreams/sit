use super::common::git_command;
use crate::error::Result;

pub fn stage_files(paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Err("No files selected".into());
    }

    let selected_paths = path_set(paths.iter().map(|p| p.as_str()));
    let already_staged = staged_paths()?;
    let dropped: Vec<String> = already_staged
        .iter()
        .filter(|p| !selected_paths.contains(*p))
        .cloned()
        .collect();

    if !dropped.is_empty() {
        return Err(
            "Some files are already staged but were not selected in the picker. Include them in selection or unstage them first."
                .into(),
        );
    }

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

fn staged_paths() -> Result<Vec<String>> {
    let output = git_command()
        .args(["diff", "--name-only", "--cached"])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to inspect staged files: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let paths = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();

    Ok(paths)
}

fn path_set<'a>(paths: impl Iterator<Item = &'a str>) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for path in paths {
        for part in path.split(" -> ").map(str::trim).filter(|p| !p.is_empty()) {
            set.insert(part.to_owned());
        }
    }
    set
}

pub fn unstage_files(paths: &[String]) -> Result<()> {
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
