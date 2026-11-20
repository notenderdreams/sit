use crate::error::Result;
use crate::git::types::{FileChange, FileStatus};

use super::common::git_output;

/// Parse `git status --porcelain -z` (NUL-separated, paths never quoted).
pub fn get_status() -> Result<Vec<FileChange>> {
    let output = git_output(&["status", "--porcelain", "-z"])?;

    if !output.status.success() {
        return Err("Not a git repository".into());
    }

    Ok(parse_porcelain_z(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_porcelain_z(raw: &str) -> Vec<FileChange> {
    let mut changes = Vec::new();
    let mut entries = raw.split('\0');

    while let Some(entry) = entries.next() {
        let bytes = entry.as_bytes();
        if bytes.len() < 4 {
            continue;
        }
        let (index, worktree) = (bytes[0], bytes[1]);
        let mut path = entry[3..].to_string();

        // Renames/copies carry the original path as the next NUL field.
        if matches!(index, b'R' | b'C')
            && let Some(orig) = entries.next()
        {
            path = format!("{orig} -> {path}");
        }

        let status = match (index, worktree) {
            (b'?', b'?') => FileStatus::Untracked,
            (b'A', _) | (_, b'A') => FileStatus::Added,
            (b'D', _) | (_, b'D') => FileStatus::Deleted,
            (b'R', _) => FileStatus::Renamed,
            _ => FileStatus::Modified,
        };

        changes.push(FileChange { path, status });
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spaces_and_renames() {
        let raw = " M my file.txt\0R  new.rs\0old.rs\0?? x\0";
        let c = parse_porcelain_z(raw);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].path, "my file.txt");
        assert_eq!(c[1].path, "old.rs -> new.rs");
        assert!(matches!(c[1].status, FileStatus::Renamed));
        assert!(matches!(c[2].status, FileStatus::Untracked));
    }
}
