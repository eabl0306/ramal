//! Detalle de commit, busqueda, historia de un fichero, blame y comparar ramas
//! contra repositorios reales de prueba.

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

use git_rudder_core_application::{GitReadError, HistoryReader};
use git_rudder_core_domain::diff::FileStatus;
use git_rudder_core_domain::history::{MatchKind, SearchQuery};
use git_rudder_core_domain::{RepoPath, RepoRoot, Sha, TrustLevel};
use git_rudder_git::{GitCliConfig, GitCliReader};
use pretty_assertions::assert_eq;
use support::FixtureRepo;

fn reader() -> GitCliReader {
    GitCliReader::new(GitCliConfig::default())
}

fn root(repo: &FixtureRepo) -> RepoRoot {
    RepoRoot::new(repo.path())
}

fn sha(text: &str) -> Sha {
    Sha::parse(text).unwrap()
}

fn path(text: &str) -> RepoPath {
    RepoPath::parse(text).unwrap()
}

#[tokio::test]
async fn a_merge_detail_lists_every_parent_and_its_files_with_renames() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("old.txt", "a\nb\nc\n")], "Initial");
    repo.checkout_new("feature");
    repo.commit_files(&[("feature.txt", "f\n")], "Feature");
    repo.checkout("main");
    let renamed = repo.rename("old.txt", "new.txt", "Rename\n\nWith a body");
    let merge = repo.merge(&["feature"], "Merge feature");

    let detail = reader()
        .commit_detail(&root(&repo), sha(&merge))
        .await
        .unwrap();
    let rename = reader()
        .commit_detail(&root(&repo), sha(&renamed))
        .await
        .unwrap();

    assert_eq!(detail.sha.to_hex(), merge);
    assert_eq!(
        detail.parents,
        vec![sha(&renamed), sha(&repo.rev("feature"))]
    );
    assert_eq!(detail.files.len(), 1);
    assert_eq!(detail.files[0].path, "feature.txt");
    assert_eq!(rename.message, "Rename\n\nWith a body");
    assert_eq!(rename.files[0].status, FileStatus::Renamed);
    assert_eq!(rename.files[0].old_path.as_deref(), Some("old.txt"));
    assert_eq!(detail.author.name, "Ana Ruiz");
}

#[tokio::test]
async fn an_unknown_commit_is_missing() {
    let repo = FixtureRepo::with_merged_feature();

    let result = reader()
        .commit_detail(&root(&repo), sha(&"a".repeat(40)))
        .await;

    assert!(
        matches!(result, Err(GitReadError::Missing { .. })),
        "{result:?}"
    );
    assert!(matches!(
        reader()
            .resolve_commit(&root(&repo), "no-such-branch")
            .await,
        Err(GitReadError::Missing { .. })
    ));
    assert!(matches!(
        reader().resolve_commit(&root(&repo), "has space").await,
        Err(GitReadError::Missing { .. })
    ));
}

#[tokio::test]
async fn search_finds_text_authors_and_short_shas_in_the_whole_history() {
    let repo = FixtureRepo::new();
    for n in 0..300 {
        repo.commit(&format!("Routine change {n}"));
    }
    let old = repo.rev("HEAD~250");
    let query = SearchQuery::parse("routine change 49").unwrap();

    let by_text = reader().search(&root(&repo), &query, 10).await.unwrap();
    let short = &old[..8];
    let by_sha = reader()
        .search(&root(&repo), &SearchQuery::parse(short).unwrap(), 10)
        .await
        .unwrap();
    let by_author = reader()
        .search(&root(&repo), &SearchQuery::parse("ana@example").unwrap(), 3)
        .await
        .unwrap();

    assert_eq!(by_text.len(), 1, "only 'change 49' matches, not 149 or 249");
    assert_eq!(by_text[0].matched, MatchKind::Message);
    assert_eq!(by_sha[0].commit.node.sha, sha(&old));
    assert_eq!(by_sha[0].matched, MatchKind::Sha);
    assert_eq!(by_author.len(), 3);
    assert_eq!(by_author[0].matched, MatchKind::Author);
}

#[tokio::test]
async fn file_history_follows_renames_and_pages() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("first name.txt", "one\ntwo\nthree\nfour\n")], "Add");
    repo.commit_files(
        &[("first name.txt", "one\ntwo\nthree\nfour\nfive\n")],
        "Edit",
    );
    repo.commit_files(&[("other.txt", "x\n")], "Unrelated");
    repo.rename("first name.txt", "second name.txt", "Rename");

    let all = reader()
        .file_history(&root(&repo), &path("second name.txt"), 0, 10)
        .await
        .unwrap();
    let page = reader()
        .file_history(&root(&repo), &path("second name.txt"), 1, 1)
        .await
        .unwrap();

    let shape: Vec<(&str, &str, FileStatus)> = all
        .iter()
        .map(|e| (e.commit.summary.as_str(), e.path.as_str(), e.status))
        .collect();
    assert_eq!(
        shape,
        vec![
            ("Rename", "second name.txt", FileStatus::Renamed),
            ("Edit", "first name.txt", FileStatus::Modified),
            ("Add", "first name.txt", FileStatus::Added),
        ]
    );
    assert_eq!(all[0].old_path.as_deref(), Some("first name.txt"));
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].commit.summary, "Edit");
    let empty = FixtureRepo::new();
    let none = reader()
        .file_history(&root(&empty), &path("a.txt"), 0, 10)
        .await
        .unwrap();
    assert_eq!(none, Vec::new());
}

#[tokio::test]
async fn blame_works_on_a_commit_and_on_the_work_tree() {
    let repo = FixtureRepo::new();
    let first = repo.commit_files(&[("a.txt", "one\ntwo\n")], "First");
    let second = repo.commit_files(&[("a.txt", "one\nTWO\n")], "Second");
    repo.write("a.txt", "one\nTWO\nthree\n");

    let at_first = reader()
        .blame(
            &root(&repo),
            TrustLevel::Untrusted,
            &path("a.txt"),
            Some(sha(&first)),
        )
        .await
        .unwrap();
    let worktree = reader()
        .blame(&root(&repo), TrustLevel::Untrusted, &path("a.txt"), None)
        .await
        .unwrap();

    assert_eq!(at_first.lines.len(), 2);
    assert!(at_first.lines.iter().all(|line| line.sha == sha(&first)));
    let shas: Vec<Sha> = worktree.lines.iter().map(|line| line.sha).collect();
    assert_eq!(shas, vec![sha(&first), sha(&second), Sha::ZERO]);
    assert!(
        worktree
            .commits
            .iter()
            .any(git_rudder_core_domain::blame::BlameCommit::is_uncommitted)
    );
    let missing = reader()
        .blame(
            &root(&repo),
            TrustLevel::Untrusted,
            &path("nope.txt"),
            Some(sha(&first)),
        )
        .await;
    assert!(missing.is_err());
}

#[tokio::test]
async fn blame_respects_the_ignore_revs_file() {
    let repo = FixtureRepo::new();
    let first = repo.commit_files(&[("a.txt", "one\ntwo\n")], "Write");
    let format = repo.commit_files(&[("a.txt", "one \ntwo \n")], "Reformat");
    repo.commit_files(
        &[(".git-blame-ignore-revs", &format!("{format}\n"))],
        "Ignore it",
    );

    let blame = reader()
        .blame(&root(&repo), TrustLevel::Trusted, &path("a.txt"), None)
        .await
        .unwrap();

    assert!(
        blame.lines.iter().all(|line| line.sha == sha(&first)),
        "the reformat is skipped: {:?}",
        blame.lines
    );
}

#[tokio::test]
async fn comparing_branches_uses_their_merge_base() {
    let repo = FixtureRepo::new();
    let base = repo.commit_files(&[("a.txt", "a\n")], "Base");
    repo.checkout_new("feature");
    repo.commit_files(&[("f1.txt", "1\n")], "F1");
    repo.commit_files(&[("f2.txt", "2\n")], "F2");
    repo.checkout("main");
    repo.commit_files(&[("m.txt", "m\n")], "M1");
    let main = sha(&repo.rev("main"));
    let feature = sha(&repo.rev("feature"));

    let comparison = reader().compare(&root(&repo), main, feature).await.unwrap();

    assert_eq!(comparison.merge_base, Some(sha(&base)));
    let ahead: Vec<&str> = comparison
        .ahead
        .iter()
        .map(|c| c.summary.as_str())
        .collect();
    assert_eq!(ahead, vec!["F2", "F1"]);
    assert_eq!(comparison.behind.len(), 1);
    let files: Vec<&str> = comparison.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        files,
        vec!["f1.txt", "f2.txt"],
        "m.txt is not part of the feature"
    );
    let resolved = reader()
        .resolve_commit(&root(&repo), "feature")
        .await
        .unwrap();
    assert_eq!(resolved, feature);
}
