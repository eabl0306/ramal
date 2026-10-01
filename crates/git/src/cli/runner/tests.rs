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

/// Un "git" falso, para forzar casos que el git real no da (colgarse, inundar
/// stderr). Son scripts versionados con el bit de ejecucion en `fixtures/`.
///
/// Existe por un fallo real: antes se escribian en un tempdir y se lanzaban en
/// seguida. Si otro test del mismo binario hacia fork mientras el script seguia
/// abierto para escritura, el hijo heredaba ese descriptor y el exec fallaba con
/// ETXTBSY ("Text file busy"); la CI se ponia roja una vez de cada diez. Un
/// fichero que nadie abre para escribir no puede provocarlo.
#[cfg(unix)]
fn fake_git(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/cli/runner/fixtures")
        .join(name);
    assert!(path.is_file(), "missing fixture {}", path.display());
    path
}

#[cfg(unix)]
#[test]
fn the_fake_gits_are_checked_in_as_executables() {
    use std::os::unix::fs::PermissionsExt;
    for name in ["hang.sh", "flood-stderr.sh"] {
        let mode = std::fs::metadata(fake_git(name))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111, "{name} must be executable");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn a_git_that_hangs_is_killed_at_the_timeout() {
    let limits = RunLimits {
        timeout: Duration::from_millis(200),
        max_output_bytes: 1024,
    };

    let error = GitRunner::new(fake_git("hang.sh"), limits)
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap_err();

    assert!(matches!(error, RunError::TimedOut(_)), "{error:?}");
}

#[cfg(unix)]
#[tokio::test]
async fn a_flood_of_stderr_is_drained_and_truncated() {
    let output = GitRunner::new(fake_git("flood-stderr.sh"), RunLimits::default())
        .run(None, &GitCommand::new("version"))
        .await
        .unwrap();

    assert_eq!(output.code, Some(3));
    assert_eq!(output.stderr.len(), STDERR_LIMIT);
}
