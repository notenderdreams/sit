use std::path::PathBuf;
use std::process::{Command, Output};

use crate::error::Result;

pub fn get_repo_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
}

pub fn git_command() -> Command {
    let mut cmd = Command::new("git");
    if let Some(root) = get_repo_root() {
        cmd.current_dir(root);
    }
    cmd
}

/// Run `git <args>` and log the invocation under `--verbose`.
pub fn git_output(args: &[&str]) -> std::io::Result<Output> {
    let started = std::time::Instant::now();
    let output = git_command().args(args).output();
    match &output {
        Ok(o) => crate::print::debug(&format!(
            "git {} → {} ({} ms)",
            args.join(" "),
            o.status
                .code()
                .map_or_else(|| "signal".to_owned(), |c| c.to_string()),
            started.elapsed().as_millis()
        )),
        Err(e) => crate::print::debug(&format!("git {} → spawn failed: {e}", args.join(" "))),
    }
    output
}

/// Run `git <args>`; on failure return `"{what}: {stderr}"`. Returns trimmed stdout.
pub fn git_run(args: &[&str], what: &str) -> Result<String> {
    let output = git_output(args)?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = stderr.trim();
    if detail.is_empty() {
        Err(format!("{what} (git exited with {})", output.status).into())
    } else {
        Err(format!("{what}: {detail}").into())
    }
}
