#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use git_rudder_core_domain::graph::WindowError;
use git_rudder_core_domain::{GitRef, Head, RefName, RefSnapshot, RepoId, RepoRoot};

use super::{LoadGraphWindow, WindowRequest};
use crate::error::EngineError;
use crate::port::git_reader::GitReadError;
use crate::state::graph_cache::GraphCache;
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

fn linear(count: u32) -> Vec<git_rudder_core_domain::CommitRecord> {
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
    let id = registry.register(RepoRoot::new("/work/git-rudder"));
    let use_case = LoadGraphWindow::new(git.clone(), registry, Arc::new(GraphCache::new()));
    Setup { git, use_case, id }
}

fn page(offset: usize, limit: usize, snapshot: Option<&str>) -> WindowRequest {
    WindowRequest {
        offset,
        limit,
        snapshot: snapshot.map(str::to_owned),
        ..WindowRequest::default()
    }
}

#[test]
fn it_reads_up_to_one_commit_past_the_window_and_cuts_it() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(10))),
    );

    let window = block_on(s.use_case.execute(&s.id, &page(2, 3, None))).unwrap();

    let query = s.git.last_log_query().unwrap();
    assert_eq!(query.skip, 0);
    assert!(query.max_count.unwrap() >= 6, "{query:?}");
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

    let window = block_on(s.use_case.execute(&s.id, &page(0, 10, None))).unwrap();

    assert!(window.rows[0].is_head);
    assert_eq!(window.rows[0].refs[0].name.as_str(), "refs/heads/main");
    assert!(window.rows[0].refs[0].is_head);
    assert!(!window.has_more);
}

#[test]
fn an_empty_repository_gives_an_empty_window_without_reading_the_log() {
    let s = setup(FakeGitReader::new());

    let window = block_on(s.use_case.execute(&s.id, &page(0, 200, None))).unwrap();

    assert!(window.rows.is_empty(), "{:?}", window.rows);
    assert!(!window.has_more);
    assert_eq!(window.lane_count, 0);
    assert_eq!(s.git.calls().read_log, 0);
}

#[test]
fn an_unknown_repo_id_is_not_open() {
    let s = setup(FakeGitReader::new());

    let result = block_on(
        s.use_case
            .execute(&RepoId::new("repo-9"), &page(0, 10, None)),
    );

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
        let result = block_on(s.use_case.execute(&s.id, &page(0, limit, None)));
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

    let result = block_on(s.use_case.execute(&s.id, &page(0, 10, None)));

    assert_eq!(
        result,
        Err(EngineError::GitFailed {
            detail: "fatal: corrupt object".to_owned()
        })
    );
}

#[test]
fn every_window_carries_the_fingerprint_of_the_refs_it_was_read_with() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(10))),
    );

    let window = block_on(s.use_case.execute(&s.id, &page(0, 3, None))).unwrap();

    assert_eq!(window.snapshot, on_main(1).fingerprint());
}

#[test]
fn the_next_page_with_the_same_fingerprint_is_served() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(10))),
    );
    let first = block_on(s.use_case.execute(&s.id, &page(0, 3, None))).unwrap();

    let next = block_on(
        s.use_case
            .execute(&s.id, &page(3, 3, Some(&first.snapshot))),
    )
    .unwrap();

    assert_eq!(next.rows[0].commit.node.sha, sha(4));
    assert_eq!(next.snapshot, first.snapshot);
}

#[test]
fn a_page_asked_with_an_old_fingerprint_is_stale_and_skips_the_log() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(10))),
    );
    let first = block_on(s.use_case.execute(&s.id, &page(0, 3, None))).unwrap();
    // Alguien hace un commit desde la terminal: main avanza.
    s.git.set_refs(on_main(11));

    let result = block_on(
        s.use_case
            .execute(&s.id, &page(3, 3, Some(&first.snapshot))),
    );

    assert_eq!(result, Err(EngineError::StaleWindow));
    assert_eq!(s.git.calls().read_log, 1);
    assert_eq!(
        EngineError::StaleWindow.to_string(),
        "the graph changed since the first page was read"
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

#[test]
fn the_next_page_comes_from_the_cache_without_reading_git_again() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(10))),
    );
    let first = block_on(s.use_case.execute(&s.id, &page(0, 3, None))).unwrap();

    let next = block_on(
        s.use_case
            .execute(&s.id, &page(6, 3, Some(&first.snapshot))),
    )
    .unwrap();

    assert_eq!(next.rows[0].commit.node.sha, sha(7));
    assert_eq!(s.git.calls().read_log, 1, "the first read already had them");
}

#[test]
fn a_page_past_the_cache_reads_only_what_is_missing() {
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(linear(9000))),
    );
    let first = block_on(s.use_case.execute(&s.id, &page(0, 100, None))).unwrap();
    let after_first = s.git.last_log_query().unwrap();

    let far = block_on(
        s.use_case
            .execute(&s.id, &page(2900, 100, Some(&first.snapshot))),
    )
    .unwrap();

    let query = s.git.last_log_query().unwrap();
    assert_eq!(query.skip, after_first.max_count.unwrap());
    assert_eq!(far.rows[0].commit.node.sha, sha(2901));
    assert_eq!(far.rows[0].row, 2900);
    assert!(far.has_more);
}

#[test]
fn an_author_filter_reads_the_whole_history_once_and_relinks_parents() {
    let mut log = linear(6);
    for (index, record) in log.iter_mut().enumerate() {
        if index % 2 == 1 {
            "Luis".clone_into(&mut record.author.name);
            "luis@example.com".clone_into(&mut record.author.email);
        }
    }
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(on_main(1)))
            .with_log(Ok(log)),
    );
    let request = WindowRequest {
        filter: git_rudder_core_domain::graph::GraphFilter {
            author: Some("ana".to_owned()),
            ..Default::default()
        },
        ..page(0, 10, None)
    };

    let window = block_on(s.use_case.execute(&s.id, &request)).unwrap();

    assert_eq!(s.git.last_log_query().unwrap().max_count, None);
    let shas: Vec<_> = window.rows.iter().map(|row| row.commit.node.sha).collect();
    assert_eq!(shas, vec![sha(1), sha(3), sha(5)]);
    assert_eq!(window.rows[0].commit.node.parents, vec![sha(3)]);
    assert_eq!(window.rows[0].edges[0].parent_row, Some(1));
    assert_eq!(window.lane_count, 1);

    block_on(s.use_case.execute(&s.id, &request)).unwrap();
    assert_eq!(
        s.git.calls().read_log,
        1,
        "the filtered history is cached too"
    );
}

#[test]
fn hiding_a_branch_passes_the_snapshot_refs_and_drops_its_pinned_lane() {
    let main = RefName::parse("refs/heads/main").unwrap();
    let topic = RefName::parse("refs/heads/topic").unwrap();
    let refs = RefSnapshot::new(
        vec![
            GitRef {
                name: main.clone(),
                target: sha(1),
            },
            GitRef {
                name: topic,
                target: sha(5),
            },
        ],
        Head::Branch {
            name: main.clone(),
            target: Some(sha(1)),
        },
        None,
    );
    let s = setup(
        FakeGitReader::new()
            .with_refs(Ok(refs))
            .with_log(Ok(vec![commit(5, &[])])),
    );
    let request = WindowRequest {
        filter: git_rudder_core_domain::graph::GraphFilter {
            hidden: vec![main],
            ..Default::default()
        },
        ..page(0, 10, None)
    };

    let window = block_on(s.use_case.execute(&s.id, &request)).unwrap();

    let query = s.git.last_log_query().unwrap();
    assert_eq!(
        query.revisions,
        crate::port::git_reader::LogRevisions::Refs {
            names: vec![RefName::parse("refs/heads/topic").unwrap()],
            include_head: false,
        },
        "main is hidden and HEAD is on main"
    );
    assert_eq!(
        window.rows[0].lane, 0,
        "main is hidden, so its lane is not kept"
    );
}
