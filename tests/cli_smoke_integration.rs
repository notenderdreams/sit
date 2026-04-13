mod common;

use tempfile::TempDir;

use common::{init_clean_repo, run_sit};

#[test]
fn cli_root_help_works() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["--help"]);
    assert!(out.status.success());
}

#[test]
fn cli_subcommand_help_for_all_commands_works() {
    let dir = TempDir::new().expect("tempdir");
    let command_sets = vec![
        vec!["commit", "--help"],
        vec!["categories", "--help"],
        vec!["branch", "--help"],
        vec!["log", "--help"],
        vec!["init", "--help"],
        vec!["push", "--help"],
        vec!["release", "--help"],
        vec!["amend", "--help"],
        vec!["undo", "--help"],
        vec!["connect", "--help"],
    ];

    for args in command_sets {
        let out = run_sit(dir.path(), &args);
        assert!(
            out.status.success(),
            "expected success for {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn cli_alias_help_for_all_aliases_works() {
    let dir = TempDir::new().expect("tempdir");
    let alias_sets = vec![
        vec!["c", "--help"],
        vec!["cat", "--help"],
        vec!["b", "--help"],
        vec!["l", "--help"],
        vec!["p", "--help"],
        vec!["rel", "--help"],
        vec!["a", "--help"],
        vec!["u", "--help"],
        vec!["con", "--help"],
    ];

    for args in alias_sets {
        let out = run_sit(dir.path(), &args);
        assert!(
            out.status.success(),
            "expected success for {:?}: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn cli_init_creates_config_and_hooks() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["init"]);
    assert!(out.status.success());

    assert!(dir.path().join(".sit/config.toml").exists());
    assert!(dir.path().join(".sit/hooks/pre-commit").exists());
    assert!(dir.path().join(".sit/hooks/post-commit").exists());
    assert!(dir.path().join(".sit/hooks/pre-push").exists());
    assert!(dir.path().join(".sit/hooks/post-push").exists());
}

#[test]
fn cli_categories_runs_without_git_repo() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["categories"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_commit_exits_cleanly_when_repo_has_no_changes() {
    let dir = init_clean_repo();
    let out = run_sit(dir.path(), &["commit"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_default_command_exits_cleanly_when_repo_has_no_changes() {
    let dir = init_clean_repo();
    let out = run_sit(dir.path(), &[]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_category_shortcut_known_category_exits_cleanly_when_no_changes() {
    let dir = init_clean_repo();
    let out = run_sit(dir.path(), &["feat", "ship", "it"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cli_external_shortcut_unknown_command_fails_cleanly() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["definitely-not-a-command-xyz"]);
    assert!(!out.status.success());
}

#[test]
fn cli_push_fails_outside_git_repo() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["push"]);
    assert!(!out.status.success());
}

#[test]
fn cli_log_fails_outside_git_repo() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["log"]);
    assert!(!out.status.success());
}

#[test]
fn cli_connect_fails_outside_git_repo() {
    let dir = TempDir::new().expect("tempdir");
    let out = run_sit(dir.path(), &["connect", "someone", "repo"]);
    assert!(!out.status.success());
}
