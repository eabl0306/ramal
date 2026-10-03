//! Lectura de refs y de HEAD contra repositorios reales de prueba.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use git_rudder_core_application::{GitReadError, GitReader};
use git_rudder_core_domain::{Head, RefName, RefSnapshot, RepoRoot, Sha};
use git_rudder_git::{GitCliConfig, GitCliReader};
use pretty_assertions::assert_eq;
use support::FixtureRepo;

async fn read(repo: &FixtureRepo) -> RefSnapshot {
    GitCliReader::new(GitCliConfig::default())
        .read_refs(&RepoRoot::new(repo.path()))
        .await
        .unwrap()
}

fn names(snapshot: &RefSnapshot) -> Vec<&str> {
    snapshot.refs().iter().map(|r| r.name.as_str()).collect()
}

fn sha(text: &str) -> Sha {
    Sha::parse(text).unwrap()
}

#[tokio::test]
async fn branches_and_tags_point_at_their_commits() {
    let repo = FixtureRepo::with_merged_feature();
    repo.annotated_tag("v1.1", "Release 1.1");

    let snapshot = read(&repo).await;

    assert_eq!(
        names(&snapshot),
        vec![
            "refs/heads/feature",
            "refs/heads/main",
            "refs/tags/v1.0",
            "refs/tags/v1.1"
        ]
    );
    let tip = sha(&repo.rev("main"));
    let annotated = snapshot
        .refs()
        .iter()
        .find(|r| r.name.as_str() == "refs/tags/v1.1");
    assert_eq!(
        annotated.unwrap().target,
        tip,
        "an annotated tag is peeled to its commit"
    );
    assert_eq!(
        snapshot.head(),
        &Head::Branch {
            name: RefName::parse("refs/heads/main").unwrap(),
            target: Some(tip),
        }
    );
    assert!(!snapshot.is_empty());
}

#[tokio::test]
async fn a_ref_name_with_commas_is_read_whole() {
    let repo = FixtureRepo::new();
    repo.commit("Initial commit");
    repo.branch("feature/a,b");

    let snapshot = read(&repo).await;

    assert!(names(&snapshot).contains(&"refs/heads/feature/a,b"));
}

#[tokio::test]
async fn remote_branches_and_origin_head_are_read() {
    let repo = FixtureRepo::new();
    let tip = repo.commit("Initial commit");
    repo.git(&["update-ref", "refs/remotes/origin/trunk", &tip]);
    repo.git(&[
        "symbolic-ref",
        "refs/remotes/origin/HEAD",
        "refs/remotes/origin/trunk",
    ]);

    let snapshot = read(&repo).await;

    assert_eq!(
        names(&snapshot),
        vec!["refs/heads/main", "refs/remotes/origin/trunk"]
    );
    assert_eq!(snapshot.default_branch_tip(), Some(sha(&tip)));
}

#[tokio::test]
async fn a_detached_head_has_no_branch() {
    let repo = FixtureRepo::with_merged_feature();
    let first = repo.rev("main~1");
    repo.detach(&first);

    let snapshot = read(&repo).await;

    assert_eq!(
        snapshot.head(),
        &Head::Detached {
            target: sha(&first)
        }
    );
    assert!(snapshot.is_detached());
}

#[tokio::test]
async fn an_empty_repository_has_an_unborn_branch() {
    let repo = FixtureRepo::new();

    let snapshot = read(&repo).await;

    assert!(snapshot.is_empty());
    assert_eq!(snapshot.head_branch().unwrap().short_name(), "main");
    assert_eq!(snapshot.head_target(), None);
}

#[tokio::test]
async fn stash_notes_and_tags_of_trees_are_not_refs_of_the_graph() {
    let repo = FixtureRepo::new();
    repo.write("a.txt", "uno");
    repo.git(&["add", "a.txt"]);
    repo.commit("Add a");
    repo.write("a.txt", "dos");
    repo.git(&["stash", "--quiet"]);
    repo.git(&["notes", "add", "-m", "nota"]);
    let tree = repo.rev("HEAD:");
    repo.git(&["tag", "-a", "tree-tag", "-m", "tag of a tree", &tree]);

    let snapshot = read(&repo).await;

    assert_eq!(names(&snapshot), vec!["refs/heads/main"]);
}

#[tokio::test]
async fn a_folder_that_stopped_being_a_repository_fails_cleanly() {
    let repo = FixtureRepo::new();
    let outside = repo.outside();
    let reader = GitCliReader::new(GitCliConfig::default());

    let error = reader
        .read_refs(&RepoRoot::new(&outside))
        .await
        .unwrap_err();
    assert!(matches!(error, GitReadError::Failed { .. }), "{error:?}");

    let gone = RepoRoot::new(repo.path().join("gone"));
    let error = reader.read_refs(&gone).await.unwrap_err();
    assert!(
        matches!(error, GitReadError::NotARepository { .. }),
        "{error:?}"
    );
}
