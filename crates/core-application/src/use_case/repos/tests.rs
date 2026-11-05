#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git_rudder_core_domain::RepoRoot;
use git_rudder_core_domain::status::{
    BranchStatus, ChangeKind, EntryState, StatusCounts, StatusEntry, WorkingTreeStatus,
};

use super::{MAX_SCAN_DEPTH, ScanRepositories, SummarizeRepository};
use crate::error::EngineError;
use crate::port::host::TrustStore as _;
use crate::testing::{FakeGitReader, FakeRepoReader, FakeScanner, MemoryTrustStore, block_on};

fn dirty() -> WorkingTreeStatus {
    WorkingTreeStatus {
        branch: BranchStatus {
            branch: Some("main".to_owned()),
            ..BranchStatus::default()
        },
        entries: vec![StatusEntry {
            path: "a.txt".to_owned(),
            original_path: None,
            state: EntryState::Changed {
                index: ChangeKind::Unmodified,
                worktree: ChangeKind::Modified,
                submodule: None,
            },
        }],
        operation: None,
    }
}

#[test]
fn a_summary_counts_changes_and_reads_untrusted_unless_remembered() {
    let git = Arc::new(FakeGitReader::new().with_repository("/work/a", RepoRoot::new("/work/a")));
    let status = Arc::new(FakeRepoReader::new().with(Ok(dirty())));
    let store = Arc::new(MemoryTrustStore::default());
    let use_case = SummarizeRepository::new(git, status.clone(), store.clone(), 2);

    let summary = block_on(use_case.execute(Path::new("/work/a"))).unwrap();
    assert_eq!(summary.root, RepoRoot::new("/work/a"));
    assert_eq!(summary.branch.branch.as_deref(), Some("main"));
    assert_eq!(
        summary.counts,
        StatusCounts {
            unstaged: 1,
            ..StatusCounts::default()
        }
    );
    assert_eq!(
        status.last_args("read_status").as_deref(),
        Some("Untrusted")
    );

    block_on(store.trust(&RepoRoot::new("/work/a"))).unwrap();
    block_on(use_case.execute(Path::new("/work/a"))).unwrap();
    assert_eq!(status.last_args("read_status").as_deref(), Some("Trusted"));
    assert_eq!(use_case.in_flight(), 0, "every turn is given back");
    assert!(format!("{use_case:?}").contains("SummarizeRepository"));
}

#[test]
fn a_folder_that_is_not_a_repository_is_reported() {
    let use_case = SummarizeRepository::new(
        Arc::new(FakeGitReader::new()),
        Arc::new(FakeRepoReader::new()),
        Arc::new(MemoryTrustStore::failing()),
        1,
    );

    let result = block_on(use_case.execute(Path::new("/tmp/x")));

    assert!(matches!(result, Err(EngineError::NotARepository { .. })));
    assert_eq!(use_case.in_flight(), 0);
}

#[test]
fn scanning_caps_the_depth_and_deduplicates() {
    let scanner = Arc::new(FakeScanner::finding(vec![
        PathBuf::from("/work/b"),
        PathBuf::from("/work/a"),
        PathBuf::from("/work/b"),
    ]));
    let use_case = ScanRepositories::new(scanner.clone());

    let found = block_on(use_case.execute(Path::new("/work"), 99)).unwrap();

    assert_eq!(
        found,
        vec![PathBuf::from("/work/a"), PathBuf::from("/work/b")]
    );
    assert_eq!(
        scanner.asked(),
        vec![(PathBuf::from("/work"), MAX_SCAN_DEPTH)]
    );
    assert!(format!("{use_case:?}").contains("ScanRepositories"));
}
