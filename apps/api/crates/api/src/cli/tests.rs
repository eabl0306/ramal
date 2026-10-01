use std::ffi::OsString;
use std::path::PathBuf;

use super::{CliError, Command, parse};

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[test]
fn no_arguments_start_the_server() {
    assert_eq!(parse(args(&[])), Ok(Command::Serve));
}

#[test]
fn openapi_takes_the_next_argument_as_the_path() {
    assert_eq!(
        parse(args(&["--openapi", "apps/api/openapi.json"])),
        Ok(Command::WriteOpenApi(PathBuf::from(
            "apps/api/openapi.json"
        )))
    );
}

#[test]
fn openapi_without_a_path_is_rejected() {
    assert_eq!(parse(args(&["--openapi"])), Err(CliError::MissingPath));
}

#[test]
fn help_has_a_short_form() {
    assert_eq!(parse(args(&["--help"])), Ok(Command::Help));
    assert_eq!(parse(args(&["-h"])), Ok(Command::Help));
}

#[test]
fn unknown_and_extra_arguments_are_rejected() {
    assert_eq!(
        parse(args(&["--port", "1"])),
        Err(CliError::Unexpected("--port".to_owned()))
    );
    assert_eq!(
        parse(args(&["--help", "now"])),
        Err(CliError::Unexpected("now".to_owned()))
    );
}

#[test]
fn errors_explain_what_is_wrong() {
    assert_eq!(CliError::MissingPath.to_string(), "--openapi needs a path");
    assert!(
        CliError::Unexpected("x".to_owned())
            .to_string()
            .contains("\"x\"")
    );
}
