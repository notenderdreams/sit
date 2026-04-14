mod common;

use common::{init_repo_with_origin_upstream, run_sit, write_file};

#[test]
fn push_succeeds_with_existing_upstream() {
    let repo = init_repo_with_origin_upstream();
    write_file(&repo.path().join("README.md"), "hello\nmore\n");
    common::git_ok(repo.path(), &["add", "README.md"]);
    common::git_ok(repo.path(), &["commit", "-m", "update"]);

    let out = run_sit(repo.path(), &["push"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn push_is_blocked_by_pre_push_hook_failure() {
    let repo = init_repo_with_origin_upstream();
    write_file(&repo.path().join("README.md"), "hello\nmore\n");
    common::git_ok(repo.path(), &["add", "README.md"]);
    common::git_ok(repo.path(), &["commit", "-m", "update"]);

    write_file(
        &repo.path().join(".sit/hooks/pre-push"),
        "#!/usr/bin/env sh\nexit 1\n",
    );

    let out = run_sit(repo.path(), &["push"]);
    assert!(!out.status.success());
}

#[test]
fn push_succeeds_even_if_post_push_hook_fails() {
    let repo = init_repo_with_origin_upstream();
    write_file(&repo.path().join("README.md"), "hello\nmore\n");
    common::git_ok(repo.path(), &["add", "README.md"]);
    common::git_ok(repo.path(), &["commit", "-m", "update"]);

    write_file(
        &repo.path().join(".sit/hooks/post-push"),
        "#!/usr/bin/env sh\nexit 1\n",
    );

    let out = run_sit(repo.path(), &["push"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn push_hook_receives_cli_env_and_last_value_wins() {
    let repo = init_repo_with_origin_upstream();
    write_file(&repo.path().join("README.md"), "hello\nmore\n");
    common::git_ok(repo.path(), &["add", "README.md"]);
    common::git_ok(repo.path(), &["commit", "-m", "update"]);

    write_file(
        &repo.path().join(".sit/hooks/pre-push"),
        "#!/usr/bin/env sh\n[ \"$TEAM\" = \"platform\" ]\n",
    );

    let out = run_sit(
        repo.path(),
        &["push", "--env", "TEAM=alpha", "--env", "TEAM=platform"],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
