use std::path::PathBuf;
use std::process::Command;

use crate::error::Result;

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
pub fn run_hook(name: &str, kind: HookKind, env: &[(&str, &str)]) -> Result<()> {
    let Some(path) = find_hook(name) else {
        return Ok(());
    };

    crate::print::debug(&format!("hook '{name}': sh {}", path.display()));
    let started = std::time::Instant::now();

    let mut cmd = Command::new("sh");
    cmd.arg(&path).envs(env.iter().copied());
    if let Some(root) = crate::git::get_repo_root() {
        cmd.current_dir(root);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            let msg = format!("Failed to run hook '{name}': {e}");
            return match kind {
                HookKind::Pre => Err(msg.into()),
                HookKind::Post => {
                    crate::print::warn(&msg);
                    Ok(())
                }
            };
        }
    };

    for line in String::from_utf8_lossy(&output.stdout).trim_end().lines() {
        crate::print::hint(&format!("[{name}] {line}"));
    }
    for line in String::from_utf8_lossy(&output.stderr).trim_end().lines() {
        crate::print::hint(&format!("[{name}] {line}"));
    }

    crate::print::debug(&format!(
        "hook '{name}' exited {} ({} ms)",
        output
            .status
            .code()
            .map_or_else(|| "by signal".into(), |c| c.to_string()),
        started.elapsed().as_millis()
    ));

    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        match kind {
            HookKind::Pre => {
                return Err(format!("Hook '{name}' exited with status {code} — aborting").into());
            }
            HookKind::Post => {
                crate::print::warn(&format!("Hook '{name}' exited with status {code}"));
            }
        }
    }

    Ok(())
}

/// Merge base SIT-provided env with user-supplied env from CLI.
/// User values win on duplicate keys except for reserved `SIT_*` keys.
pub fn merge_hook_env(base: &[(&str, &str)], extra: &[(String, String)]) -> Vec<(String, String)> {
    let mut merged: Vec<(String, String)> = base
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();

    for (key, value) in extra {
        if key.starts_with("SIT_") {
            crate::print::warn(&format!(
                "Ignoring --env {}: SIT_* keys are reserved for runtime hook context",
                key
            ));
            continue;
        }

        let mut replaced = false;
        for (existing_key, existing_value) in &mut merged {
            if existing_key == key {
                *existing_value = value.clone();
                replaced = true;
                break;
            }
        }

        if !replaced {
            merged.push((key.clone(), value.clone()));
        }
    }

    merged
}

pub fn hook_env_refs(env: &[(String, String)]) -> Vec<(&str, &str)> {
    env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
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

pub const PRE_COMMIT_TEMPLATE: &str = r#"#!/usr/bin/env sh
# Example pre-commit hook.
# Exit with a non-zero status to abort the commit.

# echo "Running checks before commit..."
# cargo fmt -- --check
# cargo clippy -- -D warnings
"#;

pub const POST_COMMIT_TEMPLATE: &str = r#"#!/usr/bin/env sh
# Example post-commit hook.
# Non-zero exit code will not abort (commit already created).

# echo "Committed: $SIT_MESSAGE"
"#;

pub const PRE_PUSH_TEMPLATE: &str = r#"#!/usr/bin/env sh
# Example pre-push hook.
# Exit with a non-zero status to abort push.

# echo "Running tests before push..."
# cargo test
"#;

pub const POST_PUSH_TEMPLATE: &str = r#"#!/usr/bin/env sh
# Example post-push hook.
# Non-zero exit code will not abort push.

# echo "Pushed $SIT_REMOTE/$SIT_BRANCH"
"#;
