use crate::error::Result;

use super::common::git_command;

pub fn forward_command(args: &[String]) -> Result<()> {
    let output = git_command().args(args).output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    if stderr.contains("is not a git command") {
        let attempted = args.first().cloned().unwrap_or_else(|| "<none>".to_owned());
        return Err(format!("Command or category not found: {attempted}").into());
    }

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", stderr);
    }

    if output.status.success() {
        return Ok(());
    }

    Err(format!("Git command failed: {}", stderr.trim()).into())
}
