use std::process::Command;
use std::path::PathBuf;

use crate::print;

/// Clone a GitHub repository to the configured directory.
pub fn clone_repo(repo_url: &str, clone_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Extract repo name from URL: https://github.com/user/repo or github.com/user/repo or user/repo
    let repo_name = extract_repo_name(repo_url)?;
    
    // Create clone_dir if it doesn't exist
    if !clone_dir.exists() {
        print::info(&format!("Creating directory: {}", clone_dir.display()));
        std::fs::create_dir_all(clone_dir)?;
    }

    let target_path = clone_dir.join(&repo_name);

    // Check if already exists
    if target_path.exists() {
        return Err(format!(
            "Directory already exists: {}",
            target_path.display()
        )
        .into());
    }

    print::info(&format!("Cloning {} → {}", repo_url, target_path.display()));

    // Run git clone
    let output = Command::new("git")
        .args(["clone", repo_url, target_path.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Clone failed: {}", err).into());
    }

    print::success(&format!("Cloned to {}", target_path.display()));

    Ok(target_path)
}

fn extract_repo_name(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = url.trim();

    // Handle various URL formats
    let path = if url.starts_with("http://") || url.starts_with("https://") {
        // https://github.com/user/repo or https://github.com/user/repo.git
        url.split('/').last().unwrap_or("")
    } else if url.contains('/') {
        // user/repo or user/repo.git
        url.split('/').last().unwrap_or("")
    } else {
        // Just a repo name
        url
    };

    if path.is_empty() {
        return Err("Could not extract repository name from URL".into());
    }

    // Remove .git suffix if present
    let name = if path.ends_with(".git") {
        &path[..path.len() - 4]
    } else {
        path
    };

    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/user/repo").unwrap(),
            "repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/user/repo.git").unwrap(),
            "repo"
        );
        assert_eq!(extract_repo_name("user/repo").unwrap(), "repo");
        assert_eq!(extract_repo_name("user/repo.git").unwrap(), "repo");
    }
}
