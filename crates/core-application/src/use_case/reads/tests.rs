#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use git_rudder_core_domain::blame::Blame;
use git_rudder_core_domain::conflict::ConflictFile;
use git_rudder_core_domain::diff::{DiffOptions, DiffTarget, FileDiff, FileStatus, empty_text};
use git_rudder_core_domain::history::{BranchComparison, FileHistoryEntry};
use git_rudder_core_domain::refs::RefListing;
use git_rudder_core_domain::stash::{StashContents, StashEntry};
use git_rudder_core_domain::status::{ConflictCode, WorkingTreeStatus};
use git_rudder_core_domain::submodule::{Submodule, SubmoduleState};
use git_rudder_core_domain::{GitTime, RepoId, RepoPath, RepoRoot, Sha, TrustLevel};

use super::{
    BlameFile, CompareRefs, FileHistory, ListRefs, ListStashes, ListSubmodules, ReadCommitDetail,
    ReadConflict, ReadFileDiff, ReadStatus, ShowStash,
};
use crate::error::EngineError;
use crate::port::git_reader::GitReadError;
use crate::port::repo_reader::DiffRequest;
use crate::state::open_repositories::OpenRepositories;
use crate::testing::{FakeRepoReader, block_on, commit, sha};

struct Setup {
    reader: Arc<FakeRepoReader>,
    registry: Arc<OpenRepositories>,
    id: RepoId,
}

fn setup(reader: FakeRepoReader) -> Setup {
    let registry = Arc::new(OpenRepositories::new());
    let id = registry.register_with(RepoRoot::new("/work/r"), TrustLevel::Untrusted);
    Setup {
        reader: Arc::new(reader),
        registry,
        id,
    }
}

fn path(text: &str) -> RepoPath {
    RepoPath::parse(text).unwrap()
}

fn unknown() -> RepoId {
    RepoId::new("repo-ffffffffffffffff")
}

#[test]
fn status_is_read_with_the_trust_of_the_repository() {
    let s = setup(FakeRepoReader::new().with(Ok(WorkingTreeStatus::default())));
    let use_case = ReadStatus::new(s.reader.clone(), s.registry.clone());

    assert_eq!(
        block_on(use_case.execute(&s.id)).unwrap(),
        WorkingTreeStatus::default()
    );
    assert_eq!(
        s.reader.last_args("read_status").as_deref(),
        Some("Untrusted")
    );
    assert!(matches!(
        block_on(use_case.execute(&unknown())),
        Err(EngineError::RepositoryNotOpen { .. })
    ));
    assert!(format!("{use_case:?}").contains("ReadStatus"));
}

#[test]
fn a_diff_request_has_its_context_capped() {
    let diff = FileDiff {
        path: "a.txt".to_owned(),
        old_path: None,
        status: FileStatus::Modified,
        content: empty_text(),
        additions: 0,
        deletions: 0,
        truncated: false,
    };
    let s = setup(FakeRepoReader::new().with(Ok(diff.clone())));
    let use_case = ReadFileDiff::new(s.reader.clone(), s.registry.clone());
    let request = DiffRequest {
        target: DiffTarget::WorkingTree,
        path: path("a.txt"),
        old_path: None,
        options: DiffOptions {
            context_lines: 1_000_000,
            ..DiffOptions::default()
        },
    };

    assert_eq!(block_on(use_case.execute(&s.id, request)).unwrap(), diff);
    let args = s.reader.last_args("read_diff").unwrap();
    assert!(args.contains("context_lines: 1000 "), "{args}");
}

#[test]
fn commit_detail_and_comparisons_resolve_their_revisions() {
    let detail_reader = FakeRepoReader::new()
        .with::<git_rudder_core_domain::CommitDetail>(Err(GitReadError::Missing {
            what: "commit".to_owned(),
        }))
        .with(Ok(sha(7)))
        .with(Ok(BranchComparison {
            base: sha(7),
            head: sha(7),
            merge_base: Some(sha(7)),
            ahead: Vec::new(),
            behind: vec![commit(2, &[])],
            files: Vec::new(),
        }));
    let s = setup(detail_reader);

    let detail = ReadCommitDetail::new(s.reader.clone(), s.registry.clone());
    assert!(matches!(
        block_on(detail.execute(&s.id, sha(1))),
        Err(EngineError::NotFound { .. })
    ));

    let compare = CompareRefs::new(s.reader.clone(), s.registry.clone());
    let result = block_on(compare.execute(&s.id, "main", "feature")).unwrap();
    assert!(result.head_is_merged());
    assert_eq!(s.reader.calls("resolve_commit"), 2);
    assert_eq!(
        s.reader.last_args("resolve_commit").as_deref(),
        Some("feature")
    );
}

fn history_entry(id: u32) -> FileHistoryEntry {
    FileHistoryEntry {
        commit: commit(id, &[]),
        path: "a.txt".to_owned(),
        old_path: None,
        status: FileStatus::Modified,
    }
}

#[test]
fn file_history_pages_ask_one_more_to_know_if_there_is_more() {
    let s = setup(FakeRepoReader::new().with(Ok(vec![
        history_entry(1),
        history_entry(2),
        history_entry(3),
    ])));
    let use_case = FileHistory::new(s.reader.clone(), s.registry.clone());

    let page = block_on(use_case.execute(&s.id, &path("a.txt"), 10, 2)).unwrap();

    assert_eq!(page.entries.len(), 2);
    assert!(page.has_more);
    assert_eq!(
        s.reader.last_args("file_history").as_deref(),
        Some("a.txt 10 3")
    );
    assert!(matches!(
        block_on(use_case.execute(&s.id, &path("a.txt"), 0, 0)),
        Err(EngineError::InvalidWindow(_))
    ));
}

#[test]
fn blame_passes_path_revision_and_trust() {
    let s = setup(FakeRepoReader::new().with(Ok(Blame::default())));
    let use_case = BlameFile::new(s.reader.clone(), s.registry.clone());

    block_on(use_case.execute(&s.id, &path("src/a.rs"), Some(sha(3)))).unwrap();

    let args = s.reader.last_args("blame").unwrap();
    assert!(args.starts_with("Untrusted src/a.rs Some("), "{args}");
}

fn stash(index: u32, id: u32) -> StashEntry {
    StashEntry {
        index,
        sha: sha(id),
        message: format!("stash {index}"),
        branch: Some("main".to_owned()),
        created_at: GitTime::new(0, 0),
    }
}

#[test]
fn only_a_listed_stash_is_shown() {
    let s = setup(
        FakeRepoReader::new()
            .with(Ok(vec![stash(0, 40), stash(1, 41)]))
            .with(Ok(StashContents::default())),
    );
    let list = ListStashes::new(s.reader.clone(), s.registry.clone());
    let show = ShowStash::new(s.reader.clone(), s.registry.clone());

    assert_eq!(block_on(list.execute(&s.id)).unwrap().len(), 2);
    assert_eq!(
        block_on(show.execute(&s.id, sha(41))).unwrap(),
        StashContents::default()
    );
    assert!(matches!(
        block_on(show.execute(&s.id, sha(1))),
        Err(EngineError::NotFound { .. })
    ));
    assert_eq!(s.reader.calls("stash_contents"), 1);
}

#[test]
fn refs_and_submodules_come_back_sorted() {
    let submodule = |path: &str| Submodule {
        path: path.to_owned(),
        sha: Sha::ZERO,
        state: SubmoduleState::UpToDate,
        describe: None,
    };
    let s = setup(
        FakeRepoReader::new()
            .with(Ok(RefListing::default()))
            .with(Ok(vec![submodule("vendor/z"), submodule("lib/a")])),
    );

    let refs = ListRefs::new(s.reader.clone(), s.registry.clone());
    assert_eq!(
        block_on(refs.execute(&s.id)).unwrap(),
        RefListing::default()
    );

    let submodules = ListSubmodules::new(s.reader.clone(), s.registry.clone());
    let paths: Vec<String> = block_on(submodules.execute(&s.id))
        .unwrap()
        .into_iter()
        .map(|submodule| submodule.path)
        .collect();
    assert_eq!(paths, vec!["lib/a", "vendor/z"]);
}

#[test]
fn a_conflict_is_read_by_path() {
    let file = ConflictFile {
        path: "a.txt".to_owned(),
        code: ConflictCode::BothModified,
        binary: false,
        content: None,
    };
    let s = setup(FakeRepoReader::new().with(Ok(file.clone())));
    let use_case = ReadConflict::new(s.reader.clone(), s.registry.clone());

    assert_eq!(
        block_on(use_case.execute(&s.id, &path("a.txt"))).unwrap(),
        file
    );
    assert_eq!(
        s.reader.last_args("read_conflict").as_deref(),
        Some("a.txt")
    );
}

#[test]
fn a_reader_without_an_answer_fails_loudly() {
    let s = setup(FakeRepoReader::new());
    let use_case = ReadStatus::new(s.reader.clone(), s.registry.clone());

    let result = block_on(use_case.execute(&s.id));

    assert!(matches!(result, Err(EngineError::GitFailed { .. })));
    assert!(format!("{:?}", s.reader).contains("read_status"));
}
