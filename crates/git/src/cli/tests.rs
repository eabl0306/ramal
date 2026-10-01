#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use super::{GitBinary, GitCommand};

fn args(command: &GitCommand) -> Vec<String> {
    command
        .args()
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn fixed_arguments_and_counts_keep_their_order() {
    let command = GitCommand::new("log").arg("--all").max_count(201);

    assert_eq!(args(&command), vec!["log", "--all", "--max-count=201"]);
    assert_eq!(command.subcommand(), "log");
}

#[test]
fn revisions_go_after_a_single_end_of_options_marker() {
    let command = GitCommand::new("rev-parse")
        .arg("--verify")
        .revisions(["--upload-pack=touch /tmp/pwned"])
        .revisions([OsString::from("HEAD")]);

    assert_eq!(
        args(&command),
        vec![
            "rev-parse",
            "--verify",
            "--end-of-options",
            "--upload-pack=touch /tmp/pwned",
            "HEAD"
        ]
    );
}

#[test]
fn an_explicit_binary_is_used_as_is() {
    let explicit = Path::new("/opt/git/bin/git");

    assert_eq!(
        GitBinary::resolve(Some(explicit.to_path_buf())),
        Some(PathBuf::from(explicit))
    );
}

#[test]
fn the_first_executable_git_in_the_path_wins() {
    let empty = tempfile::tempdir().unwrap();
    let found = GitBinary::resolve(None).unwrap();
    let dir = found.parent().unwrap().as_os_str().to_owned();
    let mut paths = OsString::from(empty.path());
    paths.push(if cfg!(windows) { ";" } else { ":" });
    paths.push(&dir);

    assert_eq!(GitBinary::find_in(&paths), Some(found));
}

#[test]
fn a_path_without_git_finds_nothing() {
    let empty = tempfile::tempdir().unwrap();

    assert_eq!(GitBinary::find_in(empty.path().as_os_str()), None);
    assert_eq!(GitBinary::find_in(OsStr::new("")), None);
}

#[cfg(unix)]
#[test]
fn a_git_file_without_execute_permission_is_skipped() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let fake = dir.path().join("git");
    std::fs::write(&fake, "#!/bin/sh\n").unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert_eq!(GitBinary::find_in(dir.path().as_os_str()), None);
}
