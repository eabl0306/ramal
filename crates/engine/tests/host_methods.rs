//! Metodos del host de punta a punta: confianza, estado de fondo, buscar
//! repositorios en carpetas y cambiar el git.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

#[path = "../../git/tests/support/mod.rs"]
mod fixture;

use fixture::{FixtureRepo, path_text};
use git_rudder_engine::{CallContext, Engine, EngineBuilder, EngineConfig};
use git_rudder_protocol::{ErrorKind, RpcError};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

async fn host(engine: &Engine, method: &str, params: Value) -> Result<Value, RpcError> {
    engine.dispatch(method, params, CallContext::host()).await
}

fn kind(error: &RpcError) -> ErrorKind {
    error.data.unwrap().kind
}

fn risky_repo() -> FixtureRepo {
    let repo = FixtureRepo::new();
    repo.commit("Initial");
    repo.config("core.fsmonitor", "true");
    repo
}

#[tokio::test]
async fn a_risky_repo_opens_untrusted_and_trusting_is_remembered_on_disk() {
    let repo = risky_repo();
    let state = repo.outside().join("trusted.txt");
    let config = EngineConfig {
        trust_file: Some(state.clone()),
        ..EngineConfig::default()
    };
    let engine = EngineBuilder::new(config.clone()).build();
    let path = json!({ "path": path_text(&repo.path()) });

    let first = host(&engine, "repo/open", path.clone()).await.unwrap();
    let repo_id = first["repoId"].clone();
    let from_webview = engine
        .dispatch(
            "repo/trust",
            json!({ "repoId": repo_id }),
            CallContext::webview(),
        )
        .await
        .unwrap_err();
    let trusted = host(&engine, "repo/trust", json!({ "repoId": repo_id }))
        .await
        .unwrap();
    let restarted = EngineBuilder::new(config).build();
    let again = host(&restarted, "repo/open", path).await.unwrap();

    assert_eq!(first["trusted"], false);
    assert_eq!(
        first["trustFindings"],
        json!([{ "kind": "configKey", "key": "core.fsmonitor", "origin": "file:.git/config" }])
    );
    assert_eq!(
        kind(&from_webview),
        ErrorKind::Internal,
        "a webview cannot trust"
    );
    assert_eq!(trusted, json!({ "repoId": repo_id, "trusted": true }));
    assert_eq!(
        again["trusted"], true,
        "trust survives a restart on this device"
    );
    assert!(state.exists());
}

#[tokio::test]
async fn a_host_can_vouch_for_a_repository() {
    let repo = risky_repo();
    let engine = EngineBuilder::new(EngineConfig::default()).build();

    let handle = host(
        &engine,
        "repo/open",
        json!({ "path": path_text(&repo.path()), "trusted": true }),
    )
    .await
    .unwrap();

    assert_eq!(handle["trusted"], true);
    assert_eq!(handle["trustFindings"], json!([]));
}

#[tokio::test]
async fn background_summaries_and_folder_scans_work_without_opening() {
    let top = FixtureRepo::new();
    top.commit("Top");
    let nested = top.path().join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::process::Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(&nested)
        .status()
        .unwrap();
    top.write("dirty.txt", "x\n");
    let engine = EngineBuilder::new(EngineConfig::default()).build();

    let summary = host(
        &engine,
        "repos/summary",
        json!({ "path": path_text(&top.path()) }),
    )
    .await
    .unwrap();
    let scan = host(
        &engine,
        "repos/scan",
        json!({ "path": path_text(&top.path().parent().unwrap()), "maxDepth": 3 }),
    )
    .await
    .unwrap();
    let missing = host(&engine, "repos/scan", json!({ "path": "/no/such/folder" }))
        .await
        .unwrap_err();
    let webview = engine
        .dispatch(
            "repos/summary",
            json!({ "path": "/" }),
            CallContext::webview(),
        )
        .await
        .unwrap_err();

    assert_eq!(summary["branch"]["branch"], "main");
    // dirty.txt y la carpeta del repositorio anidado, como lo cuenta git.
    assert_eq!(summary["counts"]["untracked"], 2);
    let roots: Vec<&str> = scan["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert_eq!(
        roots,
        vec![
            path_text(&top.canonical_path()),
            path_text(&top.canonical_path().join("nested"))
        ]
    );
    assert_eq!(kind(&missing), ErrorKind::NotFound);
    assert_eq!(kind(&webview), ErrorKind::Internal);
}

#[tokio::test]
async fn git_can_be_switched_and_checked_again_without_restarting() {
    let engine = EngineBuilder::new(EngineConfig::default()).build();

    let bad = host(&engine, "git/configure", json!({ "path": "/no/such/git" }))
        .await
        .unwrap_err();
    let info_without_git = host(&engine, "engine/info", json!({})).await.unwrap_err();
    let back = host(&engine, "git/configure", json!({})).await.unwrap();
    let info = host(&engine, "engine/info", json!({})).await.unwrap();

    assert_eq!(kind(&bad), ErrorKind::GitNotFound);
    assert_eq!(kind(&info_without_git), ErrorKind::GitNotFound);
    assert_eq!(back["supported"], true);
    assert_eq!(info["git"]["path"], back["path"]);
}
