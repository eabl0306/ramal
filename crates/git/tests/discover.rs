//! Version de git y descubrimiento de repositorios contra git real.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use std::path::PathBuf;

use ramal_core_application::{GitReadError, GitReader, MINIMUM_GIT_VERSION};
use ramal_core_domain::RepoRoot;
use ramal_git::{GitCliConfig, GitCliReader};
use support::FixtureRepo;

fn reader() -> GitCliReader {
    GitCliReader::new(GitCliConfig::default())
}

#[tokio::test]
async fn the_installed_git_is_described_and_supported() {
    let reader = reader();

    let info = reader.version().await.unwrap();

    assert!(info.version >= MINIMUM_GIT_VERSION, "{info:?}");
    assert!(info.is_supported());
    assert!(info.path.is_file());
    assert_eq!(reader.version().await.unwrap(), info);
}

#[tokio::test]
async fn the_root_is_found_from_the_root_and_from_a_subfolder() {
    let repo = FixtureRepo::new();
    std::fs::create_dir_all(repo.path().join("src/deep")).unwrap();

    let from_root = reader().discover(&repo.path()).await.unwrap();
    let from_sub = reader()
        .discover(&repo.path().join("src/deep"))
        .await
        .unwrap();

    assert_eq!(from_root, RepoRoot::new(repo.canonical_path()));
    assert_eq!(from_sub, from_root);
}

#[tokio::test]
async fn a_plain_folder_is_not_a_repository() {
    let repo = FixtureRepo::new();
    let outside = repo.outside();

    let error = reader().discover(&outside).await.unwrap_err();

    assert_eq!(error, GitReadError::NotARepository { path: outside });
}

#[tokio::test]
async fn a_missing_path_or_a_file_is_not_a_repository() {
    let repo = FixtureRepo::new();
    repo.write("notes.txt", "hola");
    let missing = repo.path().join("missing");
    let file = repo.path().join("notes.txt");

    for path in [missing, file] {
        let error = reader().discover(&path).await.unwrap_err();
        assert_eq!(error, GitReadError::NotARepository { path });
    }
}

#[tokio::test]
async fn the_git_directory_itself_has_no_work_tree() {
    let repo = FixtureRepo::new();
    let git_dir = repo.path().join(".git");

    let error = reader().discover(&git_dir).await.unwrap_err();

    assert_eq!(error, GitReadError::NotARepository { path: git_dir });
}

#[tokio::test]
async fn a_path_that_looks_like_an_option_is_never_read_as_one() {
    let repo = FixtureRepo::new();
    let tricky = repo.path().join("--output=pwned");
    std::fs::create_dir_all(&tricky).unwrap();

    let root = reader().discover(&tricky).await.unwrap();

    assert_eq!(root, RepoRoot::new(repo.canonical_path()));
    assert!(!repo.path().join("pwned").exists());
}

#[tokio::test]
async fn without_git_every_call_reports_it_missing() {
    let config = GitCliConfig {
        binary: Some(PathBuf::from("/no/such/git")),
        ..GitCliConfig::default()
    };
    let reader = GitCliReader::new(config);
    let repo = FixtureRepo::new();

    assert_eq!(reader.version().await, Err(GitReadError::NotFound));
    assert_eq!(
        reader.discover(&repo.path()).await,
        Err(GitReadError::NotFound)
    );
}
