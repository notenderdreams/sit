use std::path::PathBuf;
use std::process::Command;

pub fn get_repo_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(PathBuf::from(path))
    } else {
        None
    }
}

pub fn git_command() -> Command {
    let mut cmd = Command::new("git");
    if let Some(root) = get_repo_root() {
        cmd.current_dir(root);
    }
    cmd
}
