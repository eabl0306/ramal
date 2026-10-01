//! Lectura del log del grafo contra repositorios reales de prueba.

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

use pretty_assertions::assert_eq;
use ramal_core_application::{GitReader, LogQuery};
use ramal_core_domain::{CommitRecord, RepoRoot, Sha};
use ramal_git::{GitCliConfig, GitCliReader};
use support::FixtureRepo;

async fn log(repo: &FixtureRepo, max_count: usize) -> Vec<CommitRecord> {
    GitCliReader::new(GitCliConfig::default())
        .read_log(&RepoRoot::new(repo.path()), &LogQuery { max_count })
        .await
        .unwrap()
}

fn sha(text: &str) -> Sha {
    Sha::parse(text).unwrap()
}

fn summaries(commits: &[CommitRecord]) -> Vec<&str> {
    commits
        .iter()
        .map(|commit| commit.summary.as_str())
        .collect()
}

#[tokio::test]
async fn a_merge_lists_its_parents_in_order_and_children_come_first() {
    let repo = FixtureRepo::with_merged_feature();
    let merge = repo.rev("main");

    let commits = log(&repo, 100).await;

    assert_eq!(commits.len(), 4);
    assert_eq!(commits[0].node.sha, sha(&merge));
    assert_eq!(
        commits[0].node.parents,
        vec![sha(&repo.rev("main^1")), sha(&repo.rev("main^2"))]
    );
    assert_eq!(commits[0].summary, "Merge branch 'feature'");
    assert_eq!(commits[3].summary, "Initial commit");
    assert!(commits[3].node.parents.is_empty());
    for (row, commit) in commits.iter().enumerate() {
        for parent in &commit.node.parents {
            let parent_row = commits.iter().position(|c| c.node.sha == *parent).unwrap();
            assert!(parent_row > row, "parents must come after their children");
        }
    }
}

#[tokio::test]
async fn author_date_and_short_sha_come_from_git() {
    let repo = FixtureRepo::new();
    let full = repo.commit("Initial commit");

    let commits = log(&repo, 10).await;

    assert_eq!(commits[0].author.name, "Ana Ruiz");
    assert_eq!(commits[0].author.email, "ana@example.com");
    assert_eq!(
        commits[0].authored_at.to_string(),
        "2026-09-30T10:01:00+02:00"
    );
    assert!(full.starts_with(commits[0].short_sha.as_str()));
}

#[tokio::test]
async fn an_octopus_merge_keeps_every_parent() {
    let repo = FixtureRepo::new();
    repo.commit("Initial commit");
    for name in ["a", "b", "c"] {
        repo.checkout_new(name);
        repo.commit(&format!("Work on {name}"));
        repo.checkout("main");
    }
    repo.merge(&["a", "b", "c"], "Merge branches a, b and c");

    let commits = log(&repo, 100).await;

    assert_eq!(commits[0].node.parents.len(), 4);
    assert_eq!(commits.len(), 5);
}

#[tokio::test]
async fn every_branch_is_read_and_max_count_cuts_the_list() {
    let repo = FixtureRepo::new();
    repo.commit("one");
    repo.checkout_new("side");
    repo.commit("side work");
    repo.checkout("main");
    repo.commit("two");
    repo.commit("three");

    assert_eq!(log(&repo, 100).await.len(), 4);
    assert_eq!(summaries(&log(&repo, 2).await), vec!["three", "two"]);
}

#[tokio::test]
async fn stashes_are_not_history() {
    let repo = FixtureRepo::new();
    repo.write("a.txt", "uno");
    repo.git(&["add", "a.txt"]);
    repo.commit("Add a");
    repo.write("a.txt", "dos");
    repo.git(&["stash", "--quiet"]);

    assert_eq!(summaries(&log(&repo, 100).await), vec!["Add a"]);
}

#[tokio::test]
async fn a_detached_head_is_part_of_the_graph() {
    let repo = FixtureRepo::new();
    repo.commit("one");
    repo.detach("HEAD");
    repo.commit("detached work");

    assert_eq!(
        summaries(&log(&repo, 100).await),
        vec!["detached work", "one"]
    );
}

#[tokio::test]
async fn subjects_keep_unicode_and_separator_characters() {
    let repo = FixtureRepo::new();
    repo.commit("Anadir zoom: ñandu, cafe \u{2014} y \u{1f} raro");

    let commits = log(&repo, 10).await;

    assert_eq!(
        commits[0].summary,
        "Anadir zoom: ñandu, cafe \u{2014} y \u{1f} raro"
    );
}

#[tokio::test]
async fn an_empty_repository_has_no_history() {
    let repo = FixtureRepo::new();

    assert!(log(&repo, 10).await.is_empty());
}
