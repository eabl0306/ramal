#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use ramal_core_domain::graph::WindowError;
use ramal_core_domain::{GitRef, Head, RefName, RefSnapshot, RepoId, RepoRoot};

use super::LoadGraphWindow;
use crate::error::EngineError;
use crate::port::git_reader::{GitReadError, LogQuery};
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{FakeGitReader, block_on, commit, sha};

fn on_main(target: u32) -> RefSnapshot {
    let main = RefName::parse("refs/heads/main").unwrap();
    RefSnapshot::new(
        vec![GitRef {
            name: main.clone(),
            target: sha(target),
        }],
        Head::Branch {
            name: main,
            target: Some(sha(target)),
        },
        None,
    )
}

fn linear(count: u32) -> Vec<ramal_core_domain::CommitRecord> {
    (1..=count)
        .map(|id| {
            if id == count {
                commit(id, &[])
            } else {
                commit(id, &[id + 1])
            }
        })
        .collect()
}

struct Setup {
    git: Arc<FakeGitReader>,
    use_case: LoadGraphWindow,
    id: RepoId,
}

fn setup(git: FakeGitReader) -> Setup {
    let git = Arc::new(git);
    let registry = Arc::new(OpenRepositories::new());
    let id = registry.register(RepoRoot::new("/work/ramal"));
    let use_case = LoadGraphWindow::new(git.clone(), registry);
    Setup { git, use_case, id }
}

#[test]
fn it_reads_up_to_one_commit_past_the_window_and_cuts_it() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(10))),
    );

    let window = block_on(s.use_case.execute(&s.id, 2, 3)).unwrap();

    assert_eq!(s.git.last_log_query(), Some(LogQuery { max_count: 6 }));
    assert_eq!(window.offset, 2);
    assert_eq!(window.rows.len(), 3);
    assert_eq!(window.rows[0].row, 2);
    assert_eq!(window.rows[0].commit.node.sha, sha(3));
    assert!(window.has_more);
    assert_eq!(window.lane_count, 1);
}

#[test]
fn the_head_commit_carries_its_branch_label() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(3))),
    );

    let window = block_on(s.use_case.execute(&s.id, 0, 10)).unwrap();

    assert!(window.rows[0].is_head);
    assert_eq!(window.rows[0].refs[0].name.as_str(), "refs/heads/main");
    assert!(window.rows[0].refs[0].is_head);
    assert!(!window.has_more);
}

#[test]
fn an_empty_repository_gives_an_empty_window_without_reading_the_log() {
    let s = setup(FakeGitReader::new());

    let window = block_on(s.use_case.execute(&s.id, 0, 200)).unwrap();

    assert!(window.rows.is_empty());
    assert!(!window.has_more);
    assert_eq!(window.lane_count, 0);
    assert_eq!(s.git.calls().read_log, 0);
}

#[test]
fn an_unknown_repo_id_is_not_open() {
    let s = setup(FakeGitReader::new());

    let result = block_on(s.use_case.execute(&RepoId::new("repo-9"), 0, 10));

    assert_eq!(
        result,
        Err(EngineError::RepositoryNotOpen {
            repo_id: RepoId::new("repo-9")
        })
    );
    assert_eq!(s.git.calls().read_refs, 0);
}

#[test]
fn a_limit_out_of_range_is_rejected_before_any_git_call() {
    let s = setup(FakeGitReader::new());

    for limit in [0, 1001] {
        let result = block_on(s.use_case.execute(&s.id, 0, limit));
        assert_eq!(
            result,
            Err(EngineError::InvalidWindow(WindowError::LimitOutOfRange {
                limit,
                max: 1000
            }))
        );
    }
    assert_eq!(s.git.calls().read_refs, 0);
    assert!(format!("{:?}", s.use_case).contains("LoadGraphWindow"));
}

#[test]
fn a_git_failure_reading_the_log_is_reported() {
    let s = setup(FakeGitReader::new().with_refs(Ok(on_main(1))).with_log(Err(
        GitReadError::Failed {
            detail: "fatal: corrupt object".to_owned(),
        },
    )));

    let result = block_on(s.use_case.execute(&s.id, 0, 10));

    assert_eq!(
        result,
        Err(EngineError::GitFailed {
            detail: "fatal: corrupt object".to_owned()
        })
    );
}

#[test]
fn engine_errors_read_well_in_the_log() {
    let not_open = EngineError::RepositoryNotOpen {
        repo_id: RepoId::new("repo-9"),
    };
    assert_eq!(not_open.to_string(), "repository repo-9 is not open");
    let window = EngineError::from(WindowError::LimitOutOfRange {
        limit: 0,
        max: 1000,
    });
    assert_eq!(
        window.to_string(),
        "invalid graph window: limit must be between 1 and 1000, got 0"
    );
}
