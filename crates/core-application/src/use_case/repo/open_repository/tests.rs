#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git_rudder_core_domain::trust::{ConfigEntry, TrustFinding, TrustInspection};
use git_rudder_core_domain::{GitRef, Head, RefName, RefSnapshot, RepoRoot, TrustLevel};

use super::{OpenDependencies, OpenRepository};
use crate::error::EngineError;
use crate::port::git_reader::{GitReadError, GitVersion, MINIMUM_GIT_VERSION};
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{
    FakeGitReader, FakeRepoReader, FakeWatcher, MemoryTrustStore, block_on, sha, version_info,
};

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
    let use_case = OpenRepository::new(Arc::new(git), registry.clone(), deps(safe_repo()));
    (block_on(use_case.execute(Path::new(path), false)), registry)
}

/// Un repositorio sin nada que pueda ejecutar codigo.
fn safe_repo() -> FakeRepoReader {
    FakeRepoReader::new().with(Ok(TrustInspection::default()))
}

fn deps(inspector: FakeRepoReader) -> OpenDependencies {
    OpenDependencies {
        inspector: Arc::new(inspector),
        trust_store: Arc::new(MemoryTrustStore::default()),
        watcher: None,
    }
}

#[test]
fn opening_a_folder_inside_a_repo_registers_its_root() {
    let git = FakeGitReader::new()
        .with_repository("/work/git-rudder/src", RepoRoot::new("/work/git-rudder"))
        .with_refs(Ok(on_main()));

    let (result, registry) = open(git, "/work/git-rudder/src");
    let opened = result.unwrap();

    assert!(opened.id.as_str().starts_with("repo-"));
    assert_eq!(opened.root, RepoRoot::new("/work/git-rudder"));
    assert_eq!(opened.refs.head_target(), Some(sha(1)));
    assert_eq!(
        registry.get(&opened.id),
        Some(RepoRoot::new("/work/git-rudder"))
    );
}

#[test]
fn an_old_git_is_rejected_before_touching_the_repo() {
    let git = Arc::new(
        FakeGitReader::new()
            .with_version(Ok(version_info(GitVersion::new(2, 33, 9))))
            .with_repository("/work/git-rudder", RepoRoot::new("/work/git-rudder")),
    );
    let use_case = OpenRepository::new(
        git.clone(),
        Arc::new(OpenRepositories::new()),
        deps(safe_repo()),
    );

    let result = block_on(use_case.execute(Path::new("/work/git-rudder"), false));

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

    let (result, _) = open(git, "/work/git-rudder");

    assert_eq!(result, Err(EngineError::GitUnavailable));
}

#[test]
fn a_git_failure_reading_refs_does_not_register_the_repo() {
    let git = FakeGitReader::new()
        .with_repository("/work/git-rudder", RepoRoot::new("/work/git-rudder"))
        .with_refs(Err(GitReadError::Failed {
            detail: "fatal: bad object".to_owned(),
        }));

    let (result, registry) = open(git, "/work/git-rudder");

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

    let (result, _) = open(git, "/work/git-rudder");

    assert!(matches!(result, Err(EngineError::GitFailed { .. })));
}

fn repo_git() -> Arc<FakeGitReader> {
    Arc::new(
        FakeGitReader::new()
            .with_repository("/work/zip", RepoRoot::new("/work/zip"))
            .with_refs(Ok(on_main())),
    )
}

fn risky() -> TrustInspection {
    TrustInspection {
        config: vec![ConfigEntry {
            key: "core.fsmonitor".to_owned(),
            origin: "file:.git/config".to_owned(),
        }],
        hooks: vec!["post-checkout".to_owned()],
    }
}

#[test]
fn a_repo_with_nothing_that_runs_code_is_trusted_without_asking() {
    let (result, registry) = open(
        FakeGitReader::new()
            .with_repository("/work/zip", RepoRoot::new("/work/zip"))
            .with_refs(Ok(on_main())),
        "/work/zip",
    );
    let opened = result.unwrap();

    assert_eq!(opened.trust, TrustLevel::Trusted);
    assert_eq!(opened.findings, Vec::new());
    assert_eq!(
        registry.require(&opened.id).unwrap().trust,
        TrustLevel::Trusted
    );
}

#[test]
fn a_repo_that_could_run_code_opens_untrusted_and_says_why() {
    let registry = Arc::new(OpenRepositories::new());
    let inspector = FakeRepoReader::new().with(Ok(risky()));
    let use_case = OpenRepository::new(repo_git(), registry.clone(), deps(inspector));

    let opened = block_on(use_case.execute(Path::new("/work/zip"), false)).unwrap();

    assert_eq!(opened.trust, TrustLevel::Untrusted);
    assert_eq!(
        opened.findings,
        vec![
            TrustFinding::ConfigKey {
                key: "core.fsmonitor".to_owned(),
                origin: "file:.git/config".to_owned(),
            },
            TrustFinding::Hook {
                name: "post-checkout".to_owned()
            },
        ]
    );
    assert_eq!(
        registry.require(&opened.id).unwrap().trust,
        TrustLevel::Untrusted
    );
}

#[test]
fn remembered_or_vouched_trust_skips_the_inspection() {
    let store = Arc::new(MemoryTrustStore::default());
    block_on(crate::port::host::TrustStore::trust(
        store.as_ref(),
        &RepoRoot::new("/work/zip"),
    ))
    .unwrap();
    let inspector = Arc::new(FakeRepoReader::new().with(Ok(risky())));
    let use_case = OpenRepository::new(
        repo_git(),
        Arc::new(OpenRepositories::new()),
        OpenDependencies {
            inspector: inspector.clone(),
            trust_store: store,
            watcher: None,
        },
    );

    let remembered = block_on(use_case.execute(Path::new("/work/zip"), false)).unwrap();
    assert_eq!(remembered.trust, TrustLevel::Trusted);

    let fresh = OpenRepository::new(
        repo_git(),
        Arc::new(OpenRepositories::new()),
        deps(risky_reader()),
    );
    let vouched = block_on(fresh.execute(Path::new("/work/zip"), true)).unwrap();
    assert_eq!(vouched.trust, TrustLevel::Trusted);
    assert_eq!(inspector.calls("inspect"), 0);
}

fn risky_reader() -> FakeRepoReader {
    FakeRepoReader::new().with(Ok(risky()))
}

#[test]
fn a_broken_trust_store_falls_back_to_inspecting() {
    let use_case = OpenRepository::new(
        repo_git(),
        Arc::new(OpenRepositories::new()),
        OpenDependencies {
            inspector: Arc::new(risky_reader()),
            trust_store: Arc::new(MemoryTrustStore::failing()),
            watcher: None,
        },
    );

    let opened = block_on(use_case.execute(Path::new("/work/zip"), false)).unwrap();

    assert_eq!(opened.trust, TrustLevel::Untrusted);
}

#[test]
fn an_opened_repo_is_watched_and_a_failing_watcher_does_not_stop_it() {
    for watcher in [FakeWatcher::default(), FakeWatcher::failing()] {
        let watcher = Arc::new(watcher);
        let use_case = OpenRepository::new(
            repo_git(),
            Arc::new(OpenRepositories::new()),
            OpenDependencies {
                watcher: Some(watcher.clone()),
                ..deps(safe_repo())
            },
        );

        let opened = block_on(use_case.execute(Path::new("/work/zip"), false)).unwrap();

        assert_eq!(watcher.watched(), vec![opened.id]);
    }
}
