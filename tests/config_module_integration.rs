mod common;

use std::fs;

use sit::config::Config;
use tempfile::TempDir;

use common::{CwdGuard, lock_cwd, write_file};

#[test]
fn config_load_parses_module_paths_and_recommends_default() {
    let _lock = lock_cwd();
    let dir = TempDir::new().expect("tempdir");

    write_file(
        &dir.path().join(".sit/config.toml"),
        r#"[commit]
template = "$type($mod): $message"
ask_description = true
auto_push = false

[modules]
git = { desc = "Git operations", path = "src/git" }
ui = { desc = "User interface", paths = ["src/ui", "src/style"] }
"#,
    );

    let _cwd = CwdGuard::change_to(dir.path());
    let cfg = Config::load();

    assert_eq!(cfg.modules.len(), 2);
    assert_eq!(cfg.modules[0].name, "git");
    assert_eq!(cfg.modules[0].paths, vec!["src/git"]);
    assert_eq!(cfg.modules[1].name, "ui");
    assert_eq!(cfg.modules[1].paths, vec!["src/ui", "src/style"]);

    let changed = vec![
        "src/ui/selectors.rs".to_string(),
        "src/style.rs".to_string(),
        "src/git/log.rs".to_string(),
    ];

    assert_eq!(cfg.recommended_module_name(&changed), Some("git"));
}

#[test]
fn recommended_module_matches_rename_paths() {
    let cfg = Config {
        commit: Default::default(),
        categories: vec![],
        modules: vec![
            sit::config::Module {
                name: "git".to_string(),
                description: "Git operations".to_string(),
                paths: vec!["src/git".to_string()],
            },
            sit::config::Module {
                name: "ui".to_string(),
                description: "UI".to_string(),
                paths: vec!["src/ui".to_string()],
            },
        ],
    };

    let changed = vec!["src/old/log.rs -> src/git/log.rs".to_string()];
    assert_eq!(cfg.recommended_module_name(&changed), Some("git"));
}

#[test]
fn local_config_is_found_from_nested_directory() {
    let _lock = lock_cwd();
    let dir = TempDir::new().expect("tempdir");

    write_file(
        &dir.path().join(".sit/config.toml"),
        r#"[modules]
ui = { desc = "User interface", path = "src/ui" }
"#,
    );

    let nested = dir.path().join("src/nested/deeper");
    fs::create_dir_all(&nested).expect("create nested dir");

    let _cwd = CwdGuard::change_to(&nested);
    let cfg = Config::load();

    assert_eq!(cfg.modules.len(), 1);
    assert_eq!(cfg.modules[0].name, "ui");
}

#[test]
fn format_subject_handles_none_and_module_placeholder_collapse() {
    let cfg = Config::default();

    let none_subject = cfg.format_subject("none", Some("ui"), "improve picker");
    assert_eq!(none_subject, "(ui): improve picker");

    let collapsed = cfg.format_subject("feat", None, "improve picker");
    assert_eq!(collapsed, "feat: improve picker");
}

#[test]
fn format_commit_appends_description_block() {
    let cfg = Config::default();
    let message = cfg.format_commit("fix", Some("git"), "handle rename", "More details");
    assert_eq!(message, "fix(git): handle rename\n\nMore details");
}

#[test]
fn recommended_module_returns_none_when_no_paths_match() {
    let cfg = Config {
        commit: Default::default(),
        categories: vec![],
        modules: vec![sit::config::Module {
            name: "ui".to_string(),
            description: "ui".to_string(),
            paths: vec!["src/ui".to_string()],
        }],
    };

    let changed = vec!["src/git/log.rs".to_string()];
    assert_eq!(cfg.recommended_module_name(&changed), None);
}

#[test]
fn recommended_module_tie_uses_config_order() {
    let cfg = Config {
        commit: Default::default(),
        categories: vec![],
        modules: vec![
            sit::config::Module {
                name: "first".to_string(),
                description: "first".to_string(),
                paths: vec!["src/shared".to_string()],
            },
            sit::config::Module {
                name: "second".to_string(),
                description: "second".to_string(),
                paths: vec!["src/shared".to_string()],
            },
        ],
    };

    let changed = vec!["src/shared/file.rs".to_string()];
    assert_eq!(cfg.recommended_module_name(&changed), Some("first"));
}
