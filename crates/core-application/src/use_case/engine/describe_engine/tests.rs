#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use super::DescribeEngine;
use crate::error::EngineError;
use crate::port::git_reader::{GitReadError, GitVersion};
use crate::testing::{FakeGitReader, block_on, version_info};

#[test]
fn it_reports_the_engine_and_git_versions() {
    let git = Arc::new(FakeGitReader::new());
    let describe = DescribeEngine::new(git.clone(), "0.1.0");

    let description = block_on(describe.execute()).unwrap();

    assert_eq!(description.engine_version, "0.1.0");
    assert_eq!(description.git.version, GitVersion::new(2, 45, 0));
    assert!(description.git.is_supported());
    assert_eq!(git.calls().version, 1);
    assert!(format!("{describe:?}").contains("0.1.0"));
}

#[test]
fn an_old_git_is_described_as_unsupported_instead_of_failing() {
    let old = version_info(GitVersion::new(2, 30, 2));
    let git = Arc::new(FakeGitReader::new().with_version(Ok(old)));

    let description = block_on(DescribeEngine::new(git, "0.1.0").execute()).unwrap();

    assert!(!description.git.is_supported());
}

#[test]
fn a_missing_git_is_an_error() {
    let git = Arc::new(FakeGitReader::new().with_version(Err(GitReadError::NotFound)));

    let result = block_on(DescribeEngine::new(git, "0.1.0").execute());

    assert_eq!(result, Err(EngineError::GitUnavailable));
}
