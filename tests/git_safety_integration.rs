mod common;

use sit::git;

use common::{CwdGuard, git_cached_names, git_ok, init_repo_with_two_files, lock_cwd};

#[test]
fn stage_files_does_not_drop_pre_staged_files() {
    let _lock = lock_cwd();
    let dir = init_repo_with_two_files();
    let repo = dir.path();

    git_ok(repo, &["add", "a.txt"]);

    let _cwd = CwdGuard::change_to(repo);
    let err = git::stage_files(&["b.txt".to_string()]).expect_err("must reject");
    let msg = err.to_string();
    assert!(
        msg.contains("already staged") || msg.contains("not selected"),
        "unexpected error: {msg}"
    );

    let staged = git_cached_names(repo);
    assert_eq!(staged, vec!["a.txt"]);
}

#[test]
fn stage_files_adds_requested_files_when_safe() {
    let _lock = lock_cwd();
    let dir = init_repo_with_two_files();
    let repo = dir.path();

    let _cwd = CwdGuard::change_to(repo);
    git::stage_files(&["a.txt".to_string(), "b.txt".to_string()]).expect("stage selected files");

    let mut staged = git_cached_names(repo);
    staged.sort();
    assert_eq!(staged, vec!["a.txt", "b.txt"]);
}

#[test]
fn stage_files_rejects_empty_selection() {
    let err = git::stage_files(&[]).expect_err("empty selection must fail");
    assert!(err.to_string().contains("No files selected"));
}

#[test]
fn unstage_files_is_noop_for_empty_input() {
    git::unstage_files(&[]).expect("empty unstage should succeed");
}

#[test]
fn has_remote_detects_existing_origin() {
    let _lock = lock_cwd();
    let dir = init_repo_with_two_files();
    let repo = dir.path();

    let _cwd = CwdGuard::change_to(repo);
    assert!(!git::has_remote("origin"));

    git_ok(
        repo,
        &["remote", "add", "origin", "https://example.com/repo.git"],
    );
    assert!(git::has_remote("origin"));
}

#[test]
fn create_tag_rejects_invalid_semver() {
    let _lock = lock_cwd();
    let dir = init_repo_with_two_files();
    let repo = dir.path();
    let _cwd = CwdGuard::change_to(repo);

    let err = git::create_tag("not-a-version").expect_err("invalid tag should fail");
    assert!(err.to_string().contains("semver-like"));
}
