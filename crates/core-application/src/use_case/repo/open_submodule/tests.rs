#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use git_rudder_core_domain::submodule::{Submodule, SubmoduleState};
use git_rudder_core_domain::trust::TrustInspection;
use git_rudder_core_domain::{RepoPath, RepoRoot, Sha};

use super::OpenSubmodule;
use crate::error::EngineError;
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{FakeGitReader, FakeRepoReader, MemoryTrustStore, block_on};
use crate::use_case::repo::{OpenDependencies, OpenRepository};

fn submodule(path: &str, state: SubmoduleState) -> Submodule {
    Submodule {
        path: path.to_owned(),
        sha: Sha::ZERO,
        state,
        describe: None,
    }
}

fn setup() -> (OpenSubmodule, git_rudder_core_domain::RepoId) {
    let git = Arc::new(FakeGitReader::new().with_repository(
        "/work/app/vendor/lib",
        RepoRoot::new("/work/app/vendor/lib"),
    ));
    let reader = Arc::new(
        FakeRepoReader::new()
            .with(Ok(vec![
                submodule("vendor/lib", SubmoduleState::UpToDate),
                submodule("vendor/empty", SubmoduleState::Uninitialized),
            ]))
            .with(Ok(TrustInspection::default())),
    );
    let registry = Arc::new(OpenRepositories::new());
    let parent = registry.register(RepoRoot::new("/work/app"));
    let open = Arc::new(OpenRepository::new(
        git,
        registry.clone(),
        OpenDependencies {
            inspector: reader.clone(),
            trust_store: Arc::new(MemoryTrustStore::default()),
            watcher: None,
        },
    ));
    (OpenSubmodule::new(reader, registry, open), parent)
}

#[test]
fn an_initialized_submodule_opens_as_a_repository() {
    let (use_case, parent) = setup();

    let opened =
        block_on(use_case.execute(&parent, &RepoPath::parse("vendor/lib").unwrap())).unwrap();

    assert_eq!(opened.root, RepoRoot::new("/work/app/vendor/lib"));
    assert_ne!(opened.id, parent);
    assert!(format!("{use_case:?}").contains("OpenSubmodule"));
}

#[test]
fn only_initialized_submodules_of_the_list_can_be_opened() {
    let (use_case, parent) = setup();

    for path in ["vendor/empty", "src"] {
        let result = block_on(use_case.execute(&parent, &RepoPath::parse(path).unwrap()));
        assert!(
            matches!(result, Err(EngineError::NotFound { .. })),
            "{path}: {result:?}"
        );
    }
}
