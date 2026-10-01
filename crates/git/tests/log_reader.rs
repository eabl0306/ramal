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

#[tokio::test]
async fn commits_with_dates_or_names_that_git_accepts_do_not_break_the_log() {
    let repo = FixtureRepo::new();
    let root = repo.commit("one");
    let odd_zone = repo.raw_commit(&root, "T <t@t> 1700000000 +0090", "odd zone");
    let far = repo.raw_commit(&odd_zone, "T <t@t> 99999999999999 +0000", "far future");
    let separator = repo.raw_commit(
        &far,
        "Bad\u{1f}Name <a\u{1f}b@t> 1700000000 +0000",
        "separator in name",
    );
    let no_date = repo.raw_commit(&separator, "T <t@t> notadate", "no date");
    repo.git(&["update-ref", "refs/heads/main", &no_date]);

    let commits = log(&repo, 100).await;

    assert_eq!(
        summaries(&commits),
        vec![
            "no date",
            "separator in name",
            "far future",
            "odd zone",
            "one"
        ]
    );
    assert_eq!(commits[0].authored_at.seconds(), 0);
    assert_eq!(commits[1].author.name, "Bad\u{1f}Name");
    assert_eq!(commits[1].author.email, "a\u{1f}b@t");
    assert_eq!(
        commits[2].authored_at.to_string(),
        "9999-12-31T23:59:59+00:00"
    );
    assert_eq!(commits[3].authored_at.offset_minutes(), 90);
}

#[tokio::test]
async fn refs_the_graph_cannot_label_are_not_walked() {
    let repo = FixtureRepo::new();
    let one = repo.commit("one");
    let old = repo.commit("two");
    repo.git(&[
        "commit",
        "--quiet",
        "--amend",
        "--allow-empty",
        "-m",
        "two amended",
    ]);
    let tree = repo.rev("HEAD:");
    let upstream = repo.git(&[
        "commit-tree",
        "-p",
        &one,
        "-m",
        "prefetched upstream work",
        &tree,
    ]);
    repo.git(&["update-ref", "refs/original/refs/heads/main", &old]);
    repo.git(&["update-ref", "refs/prefetch/remotes/origin/main", &upstream]);
    repo.git(&["update-ref", "refs/jj/keep/1", &upstream]);

    assert_eq!(
        summaries(&log(&repo, 100).await),
        vec!["two amended", "one"]
    );
}

#[tokio::test]
async fn an_unborn_head_next_to_other_branches_still_reads_them() {
    let repo = FixtureRepo::new();
    repo.commit("one");
    repo.git(&["checkout", "--quiet", "--orphan", "fresh"]);

    assert_eq!(summaries(&log(&repo, 100).await), vec!["one"]);
}
