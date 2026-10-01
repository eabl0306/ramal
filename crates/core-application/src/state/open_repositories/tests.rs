#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use ramal_core_domain::{RepoId, RepoRoot};

use super::OpenRepositories;

#[test]
fn registering_a_root_issues_a_fresh_id() {
    let registry = OpenRepositories::new();
    assert!(registry.is_empty());

    let first = registry.register(RepoRoot::new("/work/a"));
    let second = registry.register(RepoRoot::new("/work/b"));

    assert_eq!(first.as_str(), "repo-1");
    assert_eq!(second.as_str(), "repo-2");
    assert_eq!(registry.len(), 2);
    assert_eq!(registry.get(&first).unwrap(), RepoRoot::new("/work/a"));
}

#[test]
fn opening_the_same_root_twice_returns_the_same_id() {
    let registry = OpenRepositories::new();

    let first = registry.register(RepoRoot::new("/work/a"));
    let again = registry.register(RepoRoot::new("/work/a"));

    assert_eq!(first, again);
    assert_eq!(registry.len(), 1);
}

#[test]
fn an_unknown_id_is_not_open() {
    let registry = OpenRepositories::new();
    registry.register(RepoRoot::new("/work/a"));

    assert_eq!(registry.get(&RepoId::new("repo-9")), None);
}

#[test]
fn a_poisoned_lock_is_recovered() {
    let registry = std::sync::Arc::new(OpenRepositories::new());
    let shared = std::sync::Arc::clone(&registry);
    let crashed = std::panic::catch_unwind(move || {
        let _guard = shared.inner.lock().unwrap();
        std::panic::resume_unwind(Box::new("poison the lock"));
    });

    assert!(crashed.is_err());
    assert!(registry.inner.is_poisoned());
    assert_eq!(
        registry.register(RepoRoot::new("/work/a")).as_str(),
        "repo-1"
    );
}
