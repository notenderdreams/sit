use std::path::PathBuf;
use std::process::Command;

/// Whether a hook failure should abort the current operation.
pub enum HookKind {
    /// Non-zero exit code aborts the operation with an error.
    Pre,
    /// Non-zero exit code prints a warning but continues.
    Post,
}

/// Run the named hook from `.sit/hooks/`, if it exists.
///
/// Scripts are always executed via `sh` so they don't need the execute bit set
/// and don't need a shebang line.  `env` is a list of `(KEY, value)` pairs
/// exposed to the hook process.
pub fn run_hook(
    name: &str,
    kind: HookKind,
    env: &[(&str, &str)],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = find_hook(name) else {
        return Ok(());
    };

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    let output = Command::new(&shell)
        .arg(&path)
        .envs(env.iter().copied())
        .output()
        .map_err(|e| format!("Failed to run hook '{name}': {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        for line in stdout.trim().lines() {
            crate::print::hint(&format!("[{name}] {line}"));
        }
    }
    if !stderr.trim().is_empty() {
        for line in stderr.trim().lines() {
            crate::print::hint(&format!("[{name}] {line}"));
        }
    }

    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        match kind {
            HookKind::Pre => {
                return Err(format!("Hook '{name}' exited with status {code} — aborting").into());
            }
            HookKind::Post => {
                crate::print::hint(&format!("Warning: hook '{name}' exited with status {code}"));
            }
        }
    }

    Ok(())
}

fn find_hook(name: &str) -> Option<PathBuf> {
    let root = crate::git::get_repo_root()?;
    let candidates = [
        root.join(".sit/hooks").join(name),
        root.join(".sit/hooks").join(format!("{name}.sh")),
    ];
    for path in &candidates {
        if path.is_file() {
            return Some(path.to_path_buf());
        }
    }
    None
}
