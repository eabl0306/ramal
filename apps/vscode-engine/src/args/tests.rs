#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::ffi::OsString;
use std::path::PathBuf;

use super::{Args, ArgsError, Command};

fn parse(arguments: &[&str]) -> Result<Command, ArgsError> {
    Args::parse(arguments.iter().map(OsString::from))
}

#[test]
fn without_arguments_it_serves_with_the_defaults() {
    assert_eq!(parse(&[]), Ok(Command::Serve(Args::default())));
    assert_eq!(Args::default().log_level, "info");
}

#[test]
fn git_path_and_log_level_take_a_separate_or_inline_value() {
    let separate = parse(&["--git-path", "/opt/git", "--log-level", "debug"]).unwrap();
    let inline = parse(&["--git-path=/opt/git", "--log-level=debug"]).unwrap();
    let expected = Command::Serve(Args {
        git_path: Some(PathBuf::from("/opt/git")),
        log_level: "debug".to_owned(),
    });

    assert_eq!(separate, expected);
    assert_eq!(inline, expected);
}

#[test]
fn version_wins_over_everything_else() {
    assert_eq!(
        parse(&["--log-level", "trace", "--version"]),
        Ok(Command::Version)
    );
    assert_eq!(parse(&["-V"]), Ok(Command::Version));
}

#[test]
fn unknown_or_incomplete_arguments_are_errors() {
    assert_eq!(
        parse(&["--stdio"]),
        Err(ArgsError::Unknown("--stdio".to_owned()))
    );
    assert_eq!(
        parse(&["--git-path"]),
        Err(ArgsError::MissingValue("--git-path"))
    );
    assert_eq!(
        parse(&["--log-level"]).unwrap_err().to_string(),
        "--log-level needs a value"
    );
}

#[cfg(unix)]
#[test]
fn a_log_level_that_is_not_utf8_is_rejected() {
    use std::os::unix::ffi::OsStringExt;
    let arguments = vec![
        OsString::from("--log-level"),
        OsString::from_vec(vec![0xff]),
    ];

    assert_eq!(
        Args::parse(arguments),
        Err(ArgsError::NotUtf8("--log-level"))
    );
}
