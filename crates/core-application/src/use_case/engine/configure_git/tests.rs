#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::path::PathBuf;
use std::sync::Arc;

use super::ConfigureGit;
use crate::error::EngineError;
use crate::port::git_reader::GitVersion;
use crate::testing::{FakeGitSetup, block_on};

#[test]
fn a_new_path_is_checked_again_and_a_bad_one_is_git_not_found() {
    let setup = Arc::new(FakeGitSetup::default());
    let use_case = ConfigureGit::new(setup.clone());

    let custom = block_on(use_case.execute(Some(PathBuf::from("/opt/git/bin/git")))).unwrap();
    assert_eq!(custom.path, PathBuf::from("/opt/git/bin/git"));
    assert_eq!(custom.version, GitVersion::new(2, 47, 0));

    let bad = block_on(use_case.execute(Some(PathBuf::from("/opt/nothing"))));
    assert_eq!(bad, Err(EngineError::GitUnavailable));

    let from_path = block_on(use_case.execute(None)).unwrap();
    assert_eq!(from_path.version, GitVersion::new(2, 45, 0));
    assert_eq!(setup.asked().len(), 3);
    assert!(format!("{use_case:?}").contains("ConfigureGit"));
}
