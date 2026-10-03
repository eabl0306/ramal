//! `handle` y los errores estables del protocolo, con el lector de git falso.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use std::sync::Arc;

use git_rudder_core_application::testing::{FakeGitReader, version_info};
use git_rudder_core_application::{GitReadError, GitVersion};
use git_rudder_core_domain::RepoRoot;
use git_rudder_engine::{CallContext, CallOrigin, Engine, EngineBuilder, EngineConfig};
use git_rudder_protocol::{ErrorKind, RpcError, RpcOutcome, RpcRequest, RpcResponse, error_code};
use pretty_assertions::assert_eq;
use serde_json::json;

fn engine_with(git: FakeGitReader) -> Engine {
    EngineBuilder::new(EngineConfig::default())
        .with_git_reader(Arc::new(git))
        .build()
}

fn request(raw: serde_json::Value) -> RpcRequest {
    serde_json::from_value(raw).unwrap()
}

fn error_of(response: RpcResponse) -> RpcError {
    match response.outcome {
        RpcOutcome::Error(error) => error,
        RpcOutcome::Result(result) => unreachable!("expected an error, got {result}"),
    }
}

#[tokio::test]
async fn a_request_gets_a_response_with_its_id() {
    let engine = engine_with(FakeGitReader::new());
    let raw = json!({ "jsonrpc": "2.0", "id": 41, "method": "engine/info", "params": {} });

    let response = engine.handle(request(raw)).await.unwrap();

    assert_eq!(response.id.unwrap().get(), 41);
    let RpcOutcome::Result(result) = response.outcome else {
        unreachable!("engine/info must succeed with the fake");
    };
    assert_eq!(result["git"]["version"], json!("2.45.0"));
    assert_eq!(result["git"]["path"], json!("/usr/bin/git"));
}

#[tokio::test]
async fn a_notification_gets_no_response() {
    let engine = engine_with(FakeGitReader::new());
    let raw = json!({ "jsonrpc": "2.0", "method": "$/cancel", "params": { "id": 3 } });

    assert_eq!(engine.handle(request(raw)).await, None);
}

#[tokio::test]
async fn an_unknown_method_is_method_not_found() {
    let engine = engine_with(FakeGitReader::new());
    let raw = json!({ "jsonrpc": "2.0", "id": 1, "method": "repo/delete", "params": {} });

    let error = error_of(engine.handle(request(raw)).await.unwrap());

    assert_eq!(error.code, error_code::METHOD_NOT_FOUND);
    assert_eq!(error.data.unwrap().kind, ErrorKind::Internal);
}

#[tokio::test]
async fn a_webview_context_cannot_reach_host_methods_through_handle() {
    let engine = engine_with(FakeGitReader::new());
    let raw =
        json!({ "jsonrpc": "2.0", "id": 2, "method": "repo/open", "params": { "path": "/" } });

    let response = engine
        .handle_with(request(raw), CallContext::webview())
        .await;

    assert_eq!(
        error_of(response.unwrap()).code,
        error_code::METHOD_NOT_FOUND
    );
    assert_eq!(CallContext::default().origin, CallOrigin::Host);
}

#[tokio::test]
async fn engine_info_takes_no_params() {
    let engine = engine_with(FakeGitReader::new());

    let error = engine
        .dispatch(
            "engine/info",
            json!({ "verbose": true }),
            CallContext::host(),
        )
        .await
        .unwrap_err();

    assert_eq!(error.data.unwrap().kind, ErrorKind::InvalidParams);
}

#[tokio::test]
async fn a_missing_git_is_git_not_found() {
    let engine = engine_with(FakeGitReader::new().with_version(Err(GitReadError::NotFound)));

    let error = engine
        .dispatch("engine/info", json!({}), CallContext::host())
        .await
        .unwrap_err();

    assert_eq!(error.data.unwrap().kind, ErrorKind::GitNotFound);
    assert_eq!(error.code, error_code::ENGINE_ERROR);
}

#[tokio::test]
async fn an_old_git_is_described_but_cannot_open_repositories() {
    let old = Ok(version_info(GitVersion::new(2, 30, 1)));
    let git = FakeGitReader::new()
        .with_version(old)
        .with_repository("/work/git-rudder", RepoRoot::new("/work/git-rudder"));
    let engine = engine_with(git);

    let info = engine
        .dispatch("engine/info", json!({}), CallContext::host())
        .await
        .unwrap();
    let error = engine
        .dispatch(
            "repo/open",
            json!({ "path": "/work/git-rudder" }),
            CallContext::host(),
        )
        .await
        .unwrap_err();

    assert_eq!(info["git"]["supported"], json!(false));
    assert_eq!(error.data.unwrap().kind, ErrorKind::GitVersionUnsupported);
    assert_eq!(
        error.message,
        "git 2.30.1 is not supported; the minimum is 2.34.0"
    );
}

#[tokio::test]
async fn a_git_failure_keeps_its_detail_out_of_the_response() {
    let git = FakeGitReader::new()
        .with_repository("/work/git-rudder", RepoRoot::new("/work/git-rudder"))
        .with_refs(Err(GitReadError::Failed {
            detail: "fatal: secret path /home/ana/private".to_owned(),
        }));
    let engine = engine_with(git);

    let error = engine
        .dispatch(
            "repo/open",
            json!({ "path": "/work/git-rudder" }),
            CallContext::host(),
        )
        .await
        .unwrap_err();

    assert_eq!(error.data.unwrap().kind, ErrorKind::GitFailed);
    assert_eq!(error.message, "git command failed");
}

#[test]
fn the_builder_says_whether_a_custom_reader_is_in_use() {
    let builder = EngineBuilder::new(EngineConfig::default());
    assert!(format!("{builder:?}").contains("custom_git_reader: false"));
    let engine = builder
        .with_git_reader(Arc::new(FakeGitReader::new()))
        .build();
    assert!(format!("{engine:?}").starts_with("Engine"));
}
