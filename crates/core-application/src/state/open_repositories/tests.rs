#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_domain::{RepoId, RepoRoot};

use super::OpenRepositories;

#[test]
fn registering_a_root_issues_an_id_for_it() {
    let registry = OpenRepositories::new();
    assert!(registry.is_empty());

    let first = registry.register(RepoRoot::new("/work/a"));
    let second = registry.register(RepoRoot::new("/work/b"));

    assert_ne!(first, second);
    assert_eq!(registry.len(), 2);
    assert_eq!(registry.get(&first).unwrap(), RepoRoot::new("/work/a"));
    assert_eq!(registry.get(&second).unwrap(), RepoRoot::new("/work/b"));
}

#[test]
fn a_restarted_engine_gives_the_same_root_the_same_id() {
    // Un sidecar reiniciado empieza con un registro vacio y los repos se vuelven
    // a abrir en otro orden: el id no puede depender de ese orden.
    let before = OpenRepositories::new();
    let id_a = before.register(RepoRoot::new("/work/a"));
    let id_b = before.register(RepoRoot::new("/work/b"));

    let after = OpenRepositories::new();
    let again_b = after.register(RepoRoot::new("/work/b"));

    assert_eq!(again_b, id_b);
    assert_eq!(after.get(&id_a), None);
}

#[test]
fn the_id_does_not_reveal_the_path() {
    let registry = OpenRepositories::new();

    let id = registry.register(RepoRoot::new("/home/someone/secret-project"));

    assert!(id.as_str().starts_with("repo-"));
    assert!(!id.as_str().contains("secret"));
    assert_eq!(id.as_str().len(), "repo-".len() + 16);
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
    let id = registry.register(RepoRoot::new("/work/a"));
    assert_eq!(registry.get(&id), Some(RepoRoot::new("/work/a")));
}
