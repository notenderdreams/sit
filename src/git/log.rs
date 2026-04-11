use super::common::git_command;
use crate::error::Result;

pub fn log_graph() -> Result<()> {
    let output = git_command()
        .args([
            "log",
            "--color=always",
            "--graph",
            "--pretty=format:%C(yellow)%h%Creset %C(magenta)%ad%Creset %C(white)%<(30,trunc)%s %C(bold blue)%<(15,trunc)%an%Creset %C(auto)%d%Creset",
            "--date=short",
        ])
        .output()?;

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        Err(format!(
            "Git log failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}
