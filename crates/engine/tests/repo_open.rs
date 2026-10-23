//! `engine/info` y `repo/open` de punta a punta contra repositorios reales.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

#[path = "../../git/tests/support/mod.rs"]
mod fixture;

use fixture::{FixtureRepo, path_text};
use git_rudder_engine::{CallContext, ENGINE_VERSION, Engine, EngineBuilder, EngineConfig};
use git_rudder_protocol::engine::EngineInfo;
use git_rudder_protocol::repo::RepoHandle;
use git_rudder_protocol::{ErrorKind, PROTOCOL_VERSION, RpcError, error_code};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

fn engine() -> Engine {
    EngineBuilder::new(EngineConfig::default()).build()
}

async fn call(engine: &Engine, method: &str, params: Value) -> Result<Value, RpcError> {
    engine.dispatch(method, params, CallContext::host()).await
}

async fn open(engine: &Engine, path: &str) -> Result<RepoHandle, RpcError> {
    call(engine, "repo/open", json!({ "path": path }))
        .await
        .map(|value| serde_json::from_value(value).unwrap())
}

const fn kind(error: &RpcError) -> ErrorKind {
    error.data.unwrap().kind
}

#[tokio::test]
async fn engine_info_reports_the_protocol_and_a_supported_git() {
    let value = call(&engine(), "engine/info", json!({})).await.unwrap();
    let info: EngineInfo = serde_json::from_value(value).unwrap();

    assert_eq!(info.protocol_version, PROTOCOL_VERSION);
    assert_eq!(info.engine_version, ENGINE_VERSION);
    assert!(info.git.supported);
    assert!(info.git.version.starts_with("2."));
}

#[tokio::test]
async fn opening_a_repository_returns_its_handle() {
    let repo = FixtureRepo::with_merged_feature();
    let engine = engine();

    let handle = open(&engine, &path_text(&repo.path())).await.unwrap();

    assert!(handle.repo_id.starts_with("repo-"));
    assert_eq!(
        handle,
        RepoHandle {
            repo_id: handle.repo_id.clone(),
            root: path_text(&repo.canonical_path()),
            head_ref: Some("main".to_owned()),
            head_sha: Some(repo.rev("main")),
            is_detached: false,
            is_empty: false,
        }
    );
}

#[tokio::test]
async fn opening_the_same_repository_twice_keeps_its_id() {
    let repo = FixtureRepo::with_merged_feature();
    std::fs::create_dir_all(repo.path().join("docs")).unwrap();
    let engine = engine();

    let first = open(&engine, &path_text(&repo.path())).await.unwrap();
    let again = open(&engine, &path_text(&repo.path().join("docs")))
        .await
        .unwrap();

    assert_eq!(first.repo_id, again.repo_id);
}

#[tokio::test]
async fn a_restarted_engine_gives_a_repository_the_same_id() {
    // El host de VS Code vuelve a abrir los repos de sus paneles tras reiniciar el
    // sidecar, quiza en otro orden: un panel tiene que seguir con su id.
    let first_repo = FixtureRepo::with_merged_feature();
    let second_repo = FixtureRepo::with_merged_feature();
    let before = engine();
    open(&before, &path_text(&first_repo.path())).await.unwrap();
    let second = open(&before, &path_text(&second_repo.path()))
        .await
        .unwrap();

    let again = open(&engine(), &path_text(&second_repo.path()))
        .await
        .unwrap();

    assert_eq!(again.repo_id, second.repo_id);
}

#[tokio::test]
async fn an_empty_repository_has_a_branch_but_no_head_commit() {
    let repo = FixtureRepo::new();

    let handle = open(&engine(), &path_text(&repo.path())).await.unwrap();

    assert!(handle.is_empty);
    assert_eq!(handle.head_ref.as_deref(), Some("main"));
    assert_eq!(handle.head_sha, None);
}

#[tokio::test]
async fn a_detached_head_has_a_commit_but_no_branch() {
    let repo = FixtureRepo::with_merged_feature();
    repo.detach("main~1");

    let handle = open(&engine(), &path_text(&repo.path())).await.unwrap();

    assert!(handle.is_detached);
    assert_eq!(handle.head_ref, None);
    assert_eq!(handle.head_sha, Some(repo.rev("HEAD")));
}

#[tokio::test]
async fn a_plain_folder_is_not_a_repository() {
    let repo = FixtureRepo::new();

    let error = open(&engine(), &path_text(&repo.outside()))
        .await
        .unwrap_err();

    assert_eq!(kind(&error), ErrorKind::NotARepository);
    assert_eq!(error.code, error_code::ENGINE_ERROR);
}

#[tokio::test]
async fn repo_open_rejects_unknown_fields_and_wrong_shapes() {
    let engine = engine();

    for params in [
        json!({ "path": "/tmp", "repoId": "repo-1" }),
        json!({}),
        json!(["/tmp"]),
        json!("/tmp"),
    ] {
        let error = call(&engine, "repo/open", params.clone())
            .await
            .unwrap_err();
        assert_eq!(kind(&error), ErrorKind::InvalidParams, "{params}");
        assert_eq!(error.code, error_code::INVALID_PARAMS);
    }
}

#[tokio::test]
async fn webviews_cannot_open_repositories_by_path() {
    let repo = FixtureRepo::with_merged_feature();
    let params = json!({ "path": path_text(&repo.path()) });

    let error = engine()
        .dispatch("repo/open", params, CallContext::webview())
        .await
        .unwrap_err();

    assert_eq!(error.code, error_code::METHOD_NOT_FOUND);
    assert_eq!(kind(&error), ErrorKind::Internal);
}
