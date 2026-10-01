#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ramal_core_domain::{GitRef, Head, RefName, RefSnapshot, RepoRoot};

use super::OpenRepository;
use crate::error::EngineError;
use crate::port::git_reader::{GitReadError, GitVersion, MINIMUM_GIT_VERSION};
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{FakeGitReader, block_on, sha, version_info};

fn on_main() -> RefSnapshot {
    let main = RefName::parse("refs/heads/main").unwrap();
    RefSnapshot::new(
        vec![GitRef {
            name: main.clone(),
            target: sha(1),
        }],
        Head::Branch {
            name: main,
            target: Some(sha(1)),
        },
        None,
    )
}

fn open(
    git: FakeGitReader,
    path: &str,
) -> (
    Result<super::OpenedRepository, EngineError>,
    Arc<OpenRepositories>,
) {
    let registry = Arc::new(OpenRepositories::new());
    let use_case = OpenRepository::new(Arc::new(git), registry.clone());
    (block_on(use_case.execute(Path::new(path))), registry)
}

#[test]
fn opening_a_folder_inside_a_repo_registers_its_root() {
    let git = FakeGitReader::new()
        .with_repository("/work/ramal/src", RepoRoot::new("/work/ramal"))
        .with_refs(Ok(on_main()));

    let (result, registry) = open(git, "/work/ramal/src");
    let opened = result.unwrap();

    assert_eq!(opened.id.as_str(), "repo-1");
    assert_eq!(opened.root, RepoRoot::new("/work/ramal"));
    assert_eq!(opened.refs.head_target(), Some(sha(1)));
    assert_eq!(registry.get(&opened.id), Some(RepoRoot::new("/work/ramal")));
}

#[test]
fn an_old_git_is_rejected_before_touching_the_repo() {
    let git = Arc::new(
        FakeGitReader::new()
            .with_version(Ok(version_info(GitVersion::new(2, 33, 9))))
            .with_repository("/work/ramal", RepoRoot::new("/work/ramal")),
    );
    let use_case = OpenRepository::new(git.clone(), Arc::new(OpenRepositories::new()));

    let result = block_on(use_case.execute(Path::new("/work/ramal")));

    assert_eq!(
        result,
        Err(EngineError::GitVersionUnsupported {
            found: GitVersion::new(2, 33, 9),
            minimum: MINIMUM_GIT_VERSION,
        })
    );
    assert_eq!(git.calls().discover, 0);
    assert!(format!("{use_case:?}").contains("OpenRepository"));
}

#[test]
fn a_folder_outside_any_repo_is_not_a_repository() {
    let (result, registry) = open(FakeGitReader::new(), "/tmp/nothing");

    assert_eq!(
        result,
        Err(EngineError::NotARepository {
            path: PathBuf::from("/tmp/nothing")
        })
    );
    assert!(registry.is_empty());
}

#[test]
fn a_missing_git_cannot_open_anything() {
    let git = FakeGitReader::new().with_version(Err(GitReadError::NotFound));

    let (result, _) = open(git, "/work/ramal");

    assert_eq!(result, Err(EngineError::GitUnavailable));
}

#[test]
fn a_git_failure_reading_refs_does_not_register_the_repo() {
    let git = FakeGitReader::new()
        .with_repository("/work/ramal", RepoRoot::new("/work/ramal"))
        .with_refs(Err(GitReadError::Failed {
            detail: "fatal: bad object".to_owned(),
        }));

    let (result, registry) = open(git, "/work/ramal");

    assert_eq!(
        result,
        Err(EngineError::GitFailed {
            detail: "fatal: bad object".to_owned()
        })
    );
    assert!(registry.is_empty());
}

#[test]
fn a_discover_failure_is_reported_as_is() {
    let git = FakeGitReader::new().with_discover_error(GitReadError::Failed {
        detail: "dubious ownership".to_owned(),
    });

    let (result, _) = open(git, "/work/ramal");

    assert!(matches!(result, Err(EngineError::GitFailed { .. })));
}
