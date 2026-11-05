#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_core_application::TrustStore as _;
use git_rudder_core_domain::RepoRoot;

use super::FileTrustStore;

#[tokio::test]
async fn trust_survives_a_new_store_on_the_same_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state").join("trusted-repos.txt");
    let root = RepoRoot::new("/work/zip extract");

    let first = FileTrustStore::new(Some(path.clone()));
    assert!(!first.is_trusted(&root).await.unwrap());
    first.trust(&root).await.unwrap();
    assert!(first.is_trusted(&root).await.unwrap());

    let restarted = FileTrustStore::new(Some(path.clone()));
    assert!(restarted.is_trusted(&root).await.unwrap());
    assert!(
        !restarted
            .is_trusted(&RepoRoot::new("/work/other"))
            .await
            .unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "/work/zip extract\n"
    );
}

#[tokio::test]
async fn without_a_file_trust_lives_in_memory() {
    let store = FileTrustStore::new(None);
    let root = RepoRoot::new("/work/a");

    store.trust(&root).await.unwrap();

    assert!(store.is_trusted(&root).await.unwrap());
}

#[tokio::test]
async fn a_root_with_a_newline_is_remembered_but_not_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trusted.txt");
    let store = FileTrustStore::new(Some(path.clone()));
    let odd = RepoRoot::new("/work/a\nb");

    store.trust(&odd).await.unwrap();

    assert!(store.is_trusted(&odd).await.unwrap());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "");
}

#[tokio::test]
async fn an_unreadable_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileTrustStore::new(Some(dir.path().to_path_buf()));

    assert!(store.is_trusted(&RepoRoot::new("/x")).await.is_err());
}
