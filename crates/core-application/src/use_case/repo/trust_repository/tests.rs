#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use git_rudder_core_domain::{RepoId, RepoRoot, TrustLevel};

use super::TrustRepository;
use crate::error::EngineError;
use crate::port::host::TrustStore as _;
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{MemoryTrustStore, block_on};

#[test]
fn trusting_is_remembered_and_applies_at_once() {
    let store = Arc::new(MemoryTrustStore::default());
    let registry = Arc::new(OpenRepositories::new());
    let id = registry.register(RepoRoot::new("/work/zip"));
    let use_case = TrustRepository::new(store.clone(), registry.clone());

    block_on(use_case.execute(&id)).unwrap();

    assert_eq!(registry.require(&id).unwrap().trust, TrustLevel::Trusted);
    assert!(block_on(store.is_trusted(&RepoRoot::new("/work/zip"))).unwrap());
    assert!(format!("{use_case:?}").contains("TrustRepository"));
}

#[test]
fn a_store_that_cannot_save_leaves_the_repo_untrusted() {
    let registry = Arc::new(OpenRepositories::new());
    let id = registry.register(RepoRoot::new("/work/zip"));
    let use_case = TrustRepository::new(Arc::new(MemoryTrustStore::failing()), registry.clone());

    let result = block_on(use_case.execute(&id));

    assert!(matches!(result, Err(EngineError::Internal { .. })));
    assert_eq!(registry.require(&id).unwrap().trust, TrustLevel::Untrusted);
    assert!(matches!(
        block_on(use_case.execute(&RepoId::new("repo-0"))),
        Err(EngineError::RepositoryNotOpen { .. })
    ));
}
