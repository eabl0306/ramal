#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{GitRunner, RunError, RunLimits, STDERR_LIMIT};
use crate::cli::{GitBinary, GitCommand};

fn real_git() -> GitRunner {
    let binary = GitBinary::resolve(None).expect("git must be installed to run these tests");
    GitRunner::new(binary, RunLimits::default())
}

#[test]
fn every_call_gets_the_safe_environment_and_global_options() {
    let runner = real_git();
    let command = GitCommand::new("log").arg("--all");
    let process = runner.process(Some(Path::new("/work")), &command);
    let std = process.as_std();

    let args: Vec<&OsStr> = std.get_args().collect();
    assert_eq!(
        args,
        [
            "--no-pager",
            "-c",
            "core.quotepath=off",
            "-c",
            "color.ui=false",
            "log",
            "--all"
        ]
        .map(OsStr::new)
    );
    let envs: Vec<(&OsStr, Option<&OsStr>)> = std.get_envs().collect();
    for (name, value) in [
        ("LC_ALL", "C"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("GIT_OPTIONAL_LOCKS", "0"),
    ] {
        assert!(
            envs.contains(&(OsStr::new(name), Some(OsStr::new(value)))),
            "{name}"
        );
    }
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_CONFIG_PARAMETERS",
    ] {
        assert!(
            envs.contains(&(OsStr::new(name), None)),
            "{name} must be removed"
        );
    }
    assert_eq!(std.get_current_dir(), Some(Path::new("/work")));
}

#[tokio::test]
async fn a_successful_call_returns_its_output() {
    let output = real_git()
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap();

    assert!(output.success());
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("git version "));
}

#[tokio::test]
async fn a_failing_command_is_not_a_run_error() {
    let dir = tempfile::tempdir().unwrap();
    let command = GitCommand::new("rev-parse").arg("--show-toplevel");

    let output = real_git().run(Some(dir.path()), &command).await.unwrap();

    assert!(!output.success());
    assert_eq!(output.code, Some(128));
    assert!(
        output.stderr.contains("not a git repository"),
        "{}",
        output.stderr
    );
}

#[tokio::test]
async fn a_missing_working_directory_is_reported_before_spawning() {
    let missing = PathBuf::from("/definitely/not/here");

    let error = real_git()
        .run(Some(&missing), &GitCommand::new("version"))
        .await
        .unwrap_err();

    assert!(matches!(error, RunError::MissingDirectory(path) if path == missing));
}

#[tokio::test]
async fn a_missing_binary_cannot_be_launched() {
    let runner = GitRunner::new(PathBuf::from("/nope/git"), RunLimits::default());

    let error = runner
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap_err();

    assert!(matches!(error, RunError::NotFound(_)));
    assert!(error.to_string().contains("could not be launched"));
}

#[tokio::test]
async fn output_over_the_limit_is_rejected() {
    let limits = RunLimits {
        timeout: Duration::from_secs(30),
        max_output_bytes: 4,
    };
    let runner = GitRunner::new(real_git().binary().to_path_buf(), limits);

    let error = runner
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap_err();

    assert!(matches!(error, RunError::OutputTooLarge(4)));
}

/// Un "git" falso escrito como script de shell, para forzar casos que el git real
/// no da (colgarse, inundar stderr).
#[cfg(unix)]
fn fake_git(script: &str) -> (tempfile::TempDir, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("git");
    std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    (dir, path)
}

#[cfg(unix)]
#[tokio::test]
async fn a_git_that_hangs_is_killed_at_the_timeout() {
    let (_dir, path) = fake_git("exec sleep 30");
    let limits = RunLimits {
        timeout: Duration::from_millis(200),
        max_output_bytes: 1024,
    };

    let error = GitRunner::new(path, limits)
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap_err();

    assert!(matches!(error, RunError::TimedOut(_)));
}

#[cfg(unix)]
#[tokio::test]
async fn a_flood_of_stderr_is_drained_and_truncated() {
    let (_dir, path) = fake_git("head -c 200000 /dev/zero | tr '\\0' 'x' >&2\nexit 3");

    let output = GitRunner::new(path, RunLimits::default())
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap();

    assert_eq!(output.code, Some(3));
    assert_eq!(output.stderr.len(), STDERR_LIMIT);
}
