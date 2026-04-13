#![allow(dead_code)]

use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::thread;

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tempfile::TempDir;

fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
    match cwd_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

pub struct CwdGuard {
    previous: std::path::PathBuf,
}

impl CwdGuard {
    pub fn change_to(path: &Path) -> Self {
        let previous = std::env::current_dir().expect("read cwd");
        std::env::set_current_dir(path).expect("set cwd");
        Self { previous }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

pub fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, content).expect("write file");
}

pub fn run_git(repo: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .expect("run git")
}

pub fn git_ok(repo: &Path, args: &[&str]) {
    let output = run_git(repo, args);
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn run_sit(cwd: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_sit"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run sit")
}

pub fn run_sit_in_pty(cwd: &Path, args: &[&str], input: &str) -> (u32, String) {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open pty");

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_sit"));
    cmd.cwd(cwd);
    for arg in args {
        cmd.arg(arg);
    }

    let mut child = pair.slave.spawn_command(cmd).expect("spawn sit");
    drop(pair.slave);

    let mut writer = pair.master.take_writer().expect("take writer");
    writer
        .write_all(input.as_bytes())
        .expect("write input to pty");
    writer.flush().expect("flush input to pty");

    let mut reader = pair.master.try_clone_reader().expect("clone reader");
    let reader_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = reader.read_to_end(&mut buf);
        String::from_utf8_lossy(&buf).to_string()
    });

    let status = child.wait().expect("wait child");
    let output = reader_thread.join().expect("join reader thread");

    (status.exit_code(), output)
}

pub fn git_cached_names(repo: &Path) -> Vec<String> {
    let output = run_git(repo, &["diff", "--name-only", "--cached"]);
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn init_clean_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let repo = dir.path();

    git_ok(repo, &["init"]);
    git_ok(repo, &["config", "user.email", "sit-tests@example.com"]);
    git_ok(repo, &["config", "user.name", "sit tests"]);

    write_file(&repo.join("README.md"), "hello\n");
    git_ok(repo, &["add", "."]);
    git_ok(repo, &["commit", "-m", "init"]);

    dir
}

pub fn init_repo_with_two_files() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let repo = dir.path();

    git_ok(repo, &["init"]);
    git_ok(repo, &["config", "user.email", "sit-tests@example.com"]);
    git_ok(repo, &["config", "user.name", "sit tests"]);

    write_file(&repo.join("a.txt"), "one\n");
    write_file(&repo.join("b.txt"), "one\n");
    git_ok(repo, &["add", "."]);
    git_ok(repo, &["commit", "-m", "init"]);

    write_file(&repo.join("a.txt"), "two\n");
    write_file(&repo.join("b.txt"), "two\n");

    dir
}

pub fn init_repo_with_two_commits() -> TempDir {
    let dir = init_clean_repo();
    let repo = dir.path();

    write_file(&repo.join("README.md"), "hello\nsecond\n");
    git_ok(repo, &["add", "README.md"]);
    git_ok(repo, &["commit", "-m", "second"]);

    dir
}

pub fn init_repo_with_origin_upstream() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let repo = dir.path();
    let remote_dir = repo.join("_remote.git");

    git_ok(repo, &["init"]);
    git_ok(repo, &["config", "user.email", "sit-tests@example.com"]);
    git_ok(repo, &["config", "user.name", "sit tests"]);

    write_file(&repo.join("README.md"), "hello\n");
    git_ok(repo, &["add", "."]);
    git_ok(repo, &["commit", "-m", "init"]);

    git_ok(
        repo,
        &[
            "init",
            "--bare",
            remote_dir.to_str().expect("remote path utf8"),
        ],
    );
    git_ok(
        repo,
        &[
            "remote",
            "add",
            "origin",
            remote_dir.to_str().expect("remote path utf8"),
        ],
    );

    let current =
        String::from_utf8_lossy(&run_git(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).stdout)
            .trim()
            .to_string();
    git_ok(repo, &["push", "-u", "origin", &current]);

    dir
}
