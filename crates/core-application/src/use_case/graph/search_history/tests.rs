#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use git_rudder_core_domain::graph::GraphFilter;
use git_rudder_core_domain::history::MatchKind;
use git_rudder_core_domain::{GitRef, Head, RefName, RefSnapshot, RepoId, RepoRoot};

use super::{MAX_SEARCH_RESULTS, SearchHistory};
use crate::port::repo_reader::FoundCommit;
use crate::state::graph_cache::GraphCache;
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{FakeGitReader, FakeRepoReader, block_on, commit, sha};

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
    history: Arc<FakeRepoReader>,
    use_case: SearchHistory,
    id: RepoId,
}

fn setup(found: Vec<FoundCommit>) -> Setup {
    let git = Arc::new(
        FakeGitReader::new()
            .with_refs(Ok(on_main()))
            .with_log(Ok(linear(2000))),
    );
    let history = Arc::new(FakeRepoReader::new().with(Ok(found)));
    let registry = Arc::new(OpenRepositories::new());
    let id = registry.register(RepoRoot::new("/work/r"));
    let use_case = SearchHistory::new(
        git.clone(),
        history.clone(),
        registry,
        Arc::new(GraphCache::new()),
    );
    Setup {
        git,
        history,
        use_case,
        id,
    }
}

#[test]
fn a_match_far_below_the_loaded_rows_comes_with_its_row() {
    let s = setup(vec![FoundCommit {
        commit: commit(1500, &[1501]),
        matched: MatchKind::Message,
    }]);

    let hits = block_on(
        s.use_case
            .execute(&s.id, "fix", 20, &GraphFilter::default()),
    )
    .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].row, Some(1499));
    assert_eq!(hits[0].matched, MatchKind::Message);
    assert_eq!(s.history.last_args("search").as_deref(), Some("fix 20"));
    assert_eq!(s.git.last_log_query().unwrap().max_count, None);
}

#[test]
fn an_empty_search_does_not_call_git_and_the_limit_is_capped() {
    let s = setup(Vec::new());

    let none = block_on(s.use_case.execute(&s.id, "  ", 20, &GraphFilter::default())).unwrap();
    assert_eq!(none, Vec::new());
    assert_eq!(s.history.calls("search"), 0);

    let empty = block_on(
        s.use_case
            .execute(&s.id, "x", 100_000, &GraphFilter::default()),
    )
    .unwrap();
    assert_eq!(empty, Vec::new());
    assert_eq!(
        s.history.last_args("search"),
        Some(format!("x {MAX_SEARCH_RESULTS}"))
    );
    assert_eq!(
        s.git.calls().read_refs,
        0,
        "nothing found, nothing to place"
    );
    assert!(format!("{:?}", s.use_case).contains("SearchHistory"));
}

#[test]
fn a_match_outside_the_filtered_graph_has_no_row() {
    let s = setup(vec![FoundCommit {
        commit: commit(9999, &[]),
        matched: MatchKind::Sha,
    }]);

    let hits = block_on(
        s.use_case
            .execute(&s.id, "abc1", 5, &GraphFilter::default()),
    )
    .unwrap();

    assert_eq!(hits[0].row, None);
}
