use super::common::git_command;

pub fn stage_files(paths: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if paths.is_empty() {
        return Err("No files selected".into());
    }

    // Reset index first so only selected files are staged.
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
