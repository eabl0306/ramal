//! Estado del arbol de trabajo, operaciones a medias, repos vacios y HEAD
//! separado, contra repositorios reales de prueba.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use std::time::{Duration, Instant};

use git_rudder_core_application::StatusReader;
use git_rudder_core_domain::operation::{Operation, RebaseProgress};
use git_rudder_core_domain::status::{ChangeKind, ConflictCode, EntryState, WorkingTreeStatus};
use git_rudder_core_domain::{RepoRoot, Sha, TrustLevel};
use git_rudder_git::{GitCliConfig, GitCliReader};
use pretty_assertions::assert_eq;
use support::FixtureRepo;

async fn status(repo: &FixtureRepo, trust: TrustLevel) -> WorkingTreeStatus {
    GitCliReader::new(GitCliConfig::default())
        .read_status(&RepoRoot::new(repo.path()), trust)
        .await
        .unwrap()
}

fn state_of<'a>(status: &'a WorkingTreeStatus, path: &str) -> &'a EntryState {
    &status
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .unwrap()
        .state
}

const fn changed(index: ChangeKind, worktree: ChangeKind) -> EntryState {
    EntryState::Changed {
        index,
        worktree,
        submodule: None,
    }
}

#[tokio::test]
async fn every_kind_of_change_is_read_with_spaces_and_accents() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("café con leche.txt", "a\n")], "Initial commit");
    repo.write("café con leche.txt", "a\nmore\n");
    repo.write("staged and edited.txt", "1\n");
    repo.add(&["staged and edited.txt"]);
    repo.write("staged and edited.txt", "2\n");
    repo.write("dir with space/untracked ñ.md", "x\n");

    let status = status(&repo, TrustLevel::Trusted).await;

    assert_eq!(
        state_of(&status, "café con leche.txt"),
        &changed(ChangeKind::Unmodified, ChangeKind::Modified)
    );
    assert_eq!(
        state_of(&status, "staged and edited.txt"),
        &changed(ChangeKind::Added, ChangeKind::Modified)
    );
    assert_eq!(
        state_of(&status, "dir with space/untracked ñ.md"),
        &EntryState::Untracked
    );
    assert_eq!(status.branch.branch.as_deref(), Some("main"));
    assert!(status.branch.head_sha.is_some());
    assert_eq!(status.operation, None);
}

#[tokio::test]
async fn renames_and_deletions_in_the_index_are_reported() {
    let repo = FixtureRepo::new();
    repo.commit_files(
        &[("old name.txt", "same\n"), ("gone.txt", "c\n")],
        "Initial",
    );
    repo.git(&["mv", "old name.txt", "new name.txt"]);
    repo.git(&["rm", "--quiet", "gone.txt"]);
    repo.write("café.txt", "nuevo\n");

    let status = status(&repo, TrustLevel::Untrusted).await;

    let renamed = status
        .entries
        .iter()
        .find(|entry| entry.path == "new name.txt")
        .unwrap();
    assert_eq!(renamed.original_path.as_deref(), Some("old name.txt"));
    assert_eq!(
        renamed.state,
        changed(ChangeKind::Renamed, ChangeKind::Unmodified)
    );
    assert_eq!(
        state_of(&status, "gone.txt"),
        &changed(ChangeKind::Deleted, ChangeKind::Unmodified)
    );
    assert_eq!(state_of(&status, "café.txt"), &EntryState::Untracked);
    let counts = status.counts();
    assert_eq!((counts.staged, counts.untracked), (2, 1));
}

#[tokio::test]
async fn a_held_index_lock_does_not_stop_a_background_read() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "a\n")], "Initial");
    repo.write("a.txt", "b\n");
    std::fs::write(repo.path().join(".git/index.lock"), "").unwrap();

    let status = status(&repo, TrustLevel::Trusted).await;

    assert_eq!(status.counts().unstaged, 1);
    assert!(
        repo.path().join(".git/index.lock").exists(),
        "the user's lock is left alone"
    );
}

#[tokio::test]
async fn a_medium_repository_answers_in_under_200_ms() {
    let repo = FixtureRepo::new();
    let files: Vec<(String, String)> = (0..2000)
        .map(|n| {
            (
                format!("src/module{}/file{n}.rs", n % 40),
                format!("// {n}\n"),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    repo.commit_files(&refs, "Two thousand files");
    repo.write("src/module3/file3.rs", "// changed\n");
    let reader = GitCliReader::new(GitCliConfig::default());
    let root = RepoRoot::new(repo.path());
    reader
        .read_status(&root, TrustLevel::Trusted)
        .await
        .unwrap();

    let started = Instant::now();
    let status = reader
        .read_status(&root, TrustLevel::Trusted)
        .await
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(status.counts().unstaged, 1);
    assert!(elapsed < Duration::from_millis(200), "took {elapsed:?}");
}

#[tokio::test]
async fn an_empty_repository_and_a_detached_head_are_reported() {
    let empty = FixtureRepo::new();
    let unborn = status(&empty, TrustLevel::Trusted).await;
    assert!(unborn.branch.is_unborn());
    assert_eq!(unborn.branch.branch.as_deref(), Some("main"));
    assert!(unborn.is_clean());

    let repo = FixtureRepo::with_merged_feature();
    let first = repo.rev("main~1");
    repo.detach(&first);
    let detached = status(&repo, TrustLevel::Trusted).await;
    assert!(detached.branch.is_detached());
    assert_eq!(detached.branch.head_sha, Some(Sha::parse(&first).unwrap()));
}

fn conflicting_branches(repo: &FixtureRepo) {
    repo.commit_files(&[("a.txt", "base\n")], "Base");
    repo.checkout_new("feature");
    repo.commit_files(&[("a.txt", "theirs\n")], "Theirs");
    repo.checkout("main");
    repo.commit_files(&[("a.txt", "ours\n")], "Ours");
}

#[tokio::test]
async fn a_merge_in_progress_names_the_incoming_branch_and_its_conflicts() {
    let repo = FixtureRepo::new();
    conflicting_branches(&repo);
    let incoming = repo.rev("feature");
    assert!(!repo.try_git(&["merge", "--quiet", "feature"]));

    let status = status(&repo, TrustLevel::Trusted).await;

    assert_eq!(
        status.operation,
        Some(Operation::Merge {
            incoming: Some(Sha::parse(&incoming).unwrap()),
            incoming_name: Some("feature".to_owned()),
        })
    );
    assert_eq!(
        state_of(&status, "a.txt"),
        &EntryState::Unmerged {
            conflict: ConflictCode::BothModified
        }
    );
}

#[tokio::test]
async fn a_stopped_rebase_reports_its_step_and_the_commit_it_stopped_at() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "base\n")], "Base");
    repo.checkout_new("feature");
    repo.commit_files(&[("b.txt", "1\n")], "One");
    repo.commit_files(&[("a.txt", "feature\n")], "Two");
    let stopped = repo.rev("HEAD");
    repo.commit_files(&[("c.txt", "3\n")], "Three");
    repo.checkout("main");
    repo.commit_files(&[("a.txt", "main\n")], "Main");
    repo.checkout("feature");
    assert!(!repo.try_git(&["rebase", "--quiet", "--merge", "main"]));

    let status = status(&repo, TrustLevel::Trusted).await;

    let Some(Operation::Rebase(rebase)) = status.operation else {
        unreachable!("a rebase is stopped: {:?}", status.operation)
    };
    assert_eq!(rebase.branch.as_deref(), Some("feature"));
    assert_eq!(rebase.progress, Some(RebaseProgress { step: 2, total: 3 }));
    assert_eq!(rebase.stopped_at, Some(Sha::parse(&stopped).unwrap()));
    assert_eq!(rebase.onto, Some(Sha::parse(&repo.rev("main")).unwrap()));
}

#[tokio::test]
async fn cherry_pick_and_revert_in_progress_are_detected() {
    let repo = FixtureRepo::new();
    conflicting_branches(&repo);
    let theirs = repo.rev("feature");
    assert!(!repo.try_git(&["cherry-pick", "feature"]));
    let picking = status(&repo, TrustLevel::Trusted).await;
    assert_eq!(
        picking.operation,
        Some(Operation::CherryPick {
            commit: Some(Sha::parse(&theirs).unwrap())
        })
    );

    repo.git(&["cherry-pick", "--abort"]);
    repo.commit_files(&[("a.txt", "later\n")], "Later");
    let ours = repo.rev("HEAD~1");
    assert!(!repo.try_git(&["revert", "--no-edit", "HEAD~1"]));
    let reverting = status(&repo, TrustLevel::Trusted).await;
    assert_eq!(
        reverting.operation,
        Some(Operation::Revert {
            commit: Some(Sha::parse(&ours).unwrap())
        })
    );
}

#[cfg(unix)]
#[tokio::test]
async fn an_untrusted_status_runs_neither_fsmonitor_nor_clean_filters() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "a\n")], "Initial");
    let marker = repo.outside().join("ran");
    let script = repo.outside().join("evil.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\ntouch '{}'\ncat\n", marker.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let command = script.to_string_lossy().into_owned();
    repo.config("core.fsmonitor", &command);
    repo.config("filter.evil.clean", &command);
    repo.write(".gitattributes", "*.txt filter=evil\n");
    repo.write("a.txt", "changed\n");

    let status = status(&repo, TrustLevel::Untrusted).await;

    assert_eq!(status.counts().unstaged, 1);
    assert!(
        !marker.exists(),
        "an untrusted read must not run repository code"
    );

    // Control: con confianza, git si lo ejecuta. Sin esto el test pasaria tambien
    // si el script no llegara a lanzarse nunca.
    let _ = GitCliReader::new(GitCliConfig::default())
        .read_status(&RepoRoot::new(repo.path()), TrustLevel::Trusted)
        .await;
    assert!(
        marker.exists(),
        "a trusted read runs the configured programs"
    );
}
