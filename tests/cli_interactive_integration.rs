mod common;

use common::{
    init_clean_repo, init_repo_with_origin_upstream, init_repo_with_two_commits, run_git,
    run_sit_in_pty, write_file,
};

#[test]
fn interactive_branch_cancel_with_escape() {
    let repo = init_clean_repo();
    let (code, output) = run_sit_in_pty(repo.path(), &["branch"], "\u{1b}");
    assert_ne!(code, 0, "output:\n{output}");
    assert!(
        output.to_lowercase().contains("cancel"),
        "expected cancel output, got:\n{output}"
    );
}

#[test]
fn interactive_undo_confirm_yes() {
    let repo = init_repo_with_two_commits();
    let (code, output) = run_sit_in_pty(repo.path(), &["undo"], "y");
    assert_eq!(code, 0, "output:\n{output}");

    let out = run_git(repo.path(), &["rev-list", "--count", "HEAD"]);
    assert!(out.status.success());
    let count = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(count, "1");
}

#[test]
fn interactive_amend_keep_message_confirm_yes_decline_push() {
    let repo = init_clean_repo();
    let (code, output) = run_sit_in_pty(repo.path(), &["amend"], "\nyn");
    assert_eq!(code, 0, "output:\n{output}");
}

#[test]
fn interactive_release_creates_and_pushes_tag() {
    let repo = init_repo_with_origin_upstream();
    let (code, output) = run_sit_in_pty(repo.path(), &["release"], "0.1.1\n");
    assert_eq!(code, 0, "output:\n{output}");

    let out = run_git(repo.path(), &["tag", "--list", "v0.1.1"]);
    assert!(out.status.success());
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(tag, "v0.1.1");
}

#[test]
#[ignore = "Full commit PTY flow is timing-sensitive in CI terminals; run manually"]
fn interactive_category_shortcut_commit_flow_creates_commit() {
    let repo = init_clean_repo();
    write_file(&repo.path().join("README.md"), "hello\nchanged\n");

    // file picker: Enter (accept defaults)
    // description prompt: Enter (empty)
    // confirm commit: y
    // confirm push: n
    let (code, output) = run_sit_in_pty(repo.path(), &["feat", "ship"], "\r\nyn");
    assert_eq!(code, 0, "output:\n{output}");

    let out = run_git(repo.path(), &["log", "-1", "--pretty=%s"]);
    assert!(out.status.success());
    let subject = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(
        subject.contains("feat") && subject.contains("ship"),
        "unexpected subject: {subject}"
    );
}
