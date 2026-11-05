//! Las lecturas nuevas del protocolo de punta a punta, como las pide un webview,
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

#[path = "../../git/tests/support/mod.rs"]
mod fixture;

use fixture::{FixtureRepo, path_text};
use git_rudder_engine::{CallContext, Engine, EngineBuilder, EngineConfig};
use git_rudder_protocol::{ErrorKind, RpcError};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

async fn call(engine: &Engine, method: &str, params: Value) -> Result<Value, RpcError> {
    engine
        .dispatch(method, params, CallContext::webview())
        .await
}

async fn opened(repo: &FixtureRepo) -> (Engine, String) {
    let engine = EngineBuilder::new(EngineConfig::default()).build();
    let handle = engine
        .dispatch(
            "repo/open",
            json!({ "path": path_text(&repo.path()) }),
            CallContext::host(),
        )
        .await
        .unwrap();
    (engine, handle["repoId"].as_str().unwrap().to_owned())
}

fn kind(error: &RpcError) -> ErrorKind {
    error.data.unwrap().kind
}

#[tokio::test]
async fn status_reports_entries_counts_and_branch() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "a\n")], "Initial");
    repo.write("a.txt", "b\n");
    repo.write("new.txt", "n\n");
    let (engine, repo_id) = opened(&repo).await;

    let status = call(&engine, "status/read", json!({ "repoId": repo_id }))
        .await
        .unwrap();

    assert_eq!(status["branch"]["branch"], "main");
    assert_eq!(status["branch"]["isUnborn"], false);
    assert_eq!(
        status["counts"],
        json!({ "staged": 0, "unstaged": 1, "untracked": 1, "conflicted": 0 })
    );
    assert_eq!(
        status["entries"][0]["state"],
        json!({ "kind": "changed", "index": "unmodified", "worktree": "modified" })
    );
    assert_eq!(
        status["entries"][1]["state"],
        json!({ "kind": "untracked" })
    );
    assert_eq!(status.get("operation"), None);
}

#[tokio::test]
async fn a_diff_comes_with_hunks_and_bad_paths_are_rejected() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "one\n")], "Initial");
    repo.write("a.txt", "two\n");
    let (engine, repo_id) = opened(&repo).await;

    let diff = call(
        &engine,
        "diff/file",
        json!({ "repoId": repo_id, "path": "a.txt", "target": { "kind": "workingTree" } }),
    )
    .await
    .unwrap();
    let escape = call(
        &engine,
        "diff/file",
        json!({ "repoId": repo_id, "path": "../outside", "target": { "kind": "workingTree" } }),
    )
    .await
    .unwrap_err();

    assert_eq!(diff["status"], "modified");
    assert_eq!(diff["content"]["kind"], "text");
    let lines = &diff["content"]["hunks"][0]["lines"];
    assert_eq!(
        lines[0],
        json!({ "kind": "removed", "oldLine": 1, "text": "one", "words": [], "noNewlineAtEnd": false })
    );
    assert_eq!(kind(&escape), ErrorKind::InvalidParams);
}

#[tokio::test]
async fn commit_detail_refs_and_compare_answer_with_full_shas() {
    let repo = FixtureRepo::with_merged_feature();
    let (engine, repo_id) = opened(&repo).await;
    let head = repo.rev("main");

    let detail = call(
        &engine,
        "commit/detail",
        json!({ "repoId": repo_id, "sha": head }),
    )
    .await
    .unwrap();
    let refs = call(&engine, "refs/list", json!({ "repoId": repo_id }))
        .await
        .unwrap();
    let compare = call(
        &engine,
        "compare/refs",
        json!({ "repoId": repo_id, "base": "main~1", "head": "main" }),
    )
    .await
    .unwrap();
    let missing = call(
        &engine,
        "commit/detail",
        json!({ "repoId": repo_id, "sha": "f".repeat(40) }),
    )
    .await
    .unwrap_err();

    assert_eq!(detail["sha"], head.as_str());
    assert_eq!(detail["parents"].as_array().unwrap().len(), 2);
    let local: Vec<&str> = refs["local"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["name"].as_str().unwrap())
        .collect();
    assert_eq!(local, vec!["feature", "main"]);
    assert_eq!(refs["tags"][0]["name"], "v1.0");
    assert_eq!(
        compare["ahead"].as_array().unwrap().len(),
        2,
        "the merge and the feature commit"
    );
    assert_eq!(kind(&missing), ErrorKind::NotFound);
}

#[tokio::test]
async fn search_returns_rows_and_a_filtered_window_keeps_only_matches() {
    let repo = FixtureRepo::new();
    for n in 0..30 {
        repo.commit(&format!("Change number {n}"));
    }
    let (engine, repo_id) = opened(&repo).await;

    let found = call(
        &engine,
        "graph/search",
        json!({ "repoId": repo_id, "query": "number 3" }),
    )
    .await
    .unwrap();
    let only = call(
        &engine,
        "graph/window",
        json!({ "repoId": repo_id, "offset": 0, "limit": 100,
                "filter": { "only": ["refs/heads/main"], "since": "2026-09-30T10:20:00+02:00" } }),
    )
    .await
    .unwrap();
    let bad_filter = call(
        &engine,
        "graph/window",
        json!({ "repoId": repo_id, "offset": 0, "limit": 10, "filter": { "since": "ayer" } }),
    )
    .await
    .unwrap_err();

    assert_eq!(found["hits"][0]["commit"]["summary"], "Change number 3");
    assert_eq!(found["hits"][0]["row"], 26);
    assert_eq!(found["hits"][0]["matched"], "message");
    assert_eq!(
        only["rows"].as_array().unwrap().len(),
        11,
        "commits from 10:20 on"
    );
    assert_eq!(kind(&bad_filter), ErrorKind::InvalidParams);
}

#[tokio::test]
async fn stashes_history_blame_and_conflicts_answer_through_the_protocol() {
    let repo = FixtureRepo::new();
    repo.commit_files(&[("a.txt", "base\n")], "Base");
    repo.checkout_new("feature");
    repo.commit_files(&[("a.txt", "theirs\n")], "Theirs");
    repo.checkout("main");
    repo.commit_files(&[("a.txt", "ours\n")], "Ours");
    repo.write("w.txt", "wip\n");
    repo.stash("wip");
    assert!(!repo.try_git(&["merge", "--quiet", "feature"]));
    let (engine, repo_id) = opened(&repo).await;

    let stashes = call(&engine, "stash/list", json!({ "repoId": repo_id }))
        .await
        .unwrap();
    let sha = stashes["stashes"][0]["sha"].as_str().unwrap().to_owned();
    let stash = call(
        &engine,
        "stash/show",
        json!({ "repoId": repo_id, "sha": sha }),
    )
    .await
    .unwrap();
    let history = call(
        &engine,
        "history/file",
        json!({ "repoId": repo_id, "path": "a.txt", "offset": 0, "limit": 10 }),
    )
    .await
    .unwrap();
    let blame = call(
        &engine,
        "blame/file",
        json!({ "repoId": repo_id, "path": "a.txt", "revision": repo.rev("HEAD") }),
    )
    .await
    .unwrap();
    let conflict = call(
        &engine,
        "conflict/read",
        json!({ "repoId": repo_id, "path": "a.txt" }),
    )
    .await
    .unwrap();
    let status = call(&engine, "status/read", json!({ "repoId": repo_id }))
        .await
        .unwrap();

    assert_eq!(stash["untracked"][0]["path"], "w.txt");
    assert_eq!(history["entries"].as_array().unwrap().len(), 2);
    assert_eq!(history["hasMore"], false);
    assert_eq!(blame["lines"][0]["text"], "ours");
    assert_eq!(conflict["code"], "bothModified");
    assert_eq!(conflict["segments"][0]["kind"], "hunk");
    assert_eq!(conflict["segments"][0]["base"], json!(["base"]));
    assert_eq!(status["operation"]["kind"], "merge");
    assert_eq!(status["operation"]["incomingName"], "feature");
}

#[tokio::test]
async fn submodules_are_listed_and_only_the_host_opens_them() {
    let inner = FixtureRepo::new();
    inner.commit("Inner");
    let repo = FixtureRepo::new();
    repo.commit("Outer");
    repo.add_submodule(&inner, "vendor/inner");
    let (engine, repo_id) = opened(&repo).await;

    let list = call(&engine, "submodule/list", json!({ "repoId": repo_id }))
        .await
        .unwrap();
    let params = json!({ "repoId": repo_id, "path": "vendor/inner" });
    let from_webview = call(&engine, "submodule/open", params.clone())
        .await
        .unwrap_err();
    let from_host = engine
        .dispatch("submodule/open", params, CallContext::host())
        .await
        .unwrap();

    assert_eq!(list["submodules"][0]["path"], "vendor/inner");
    assert_eq!(list["submodules"][0]["state"], "upToDate");
    assert_eq!(
        kind(&from_webview),
        ErrorKind::Internal,
        "host-only looks like a missing method"
    );
    assert_ne!(from_host["repoId"], repo_id.as_str());
}

#[tokio::test]
async fn an_unknown_repo_id_is_not_open_for_every_read() {
    let engine = EngineBuilder::new(EngineConfig::default()).build();
    let repo_id = "repo-0000000000000000";
    for (method, params) in [
        ("status/read", json!({ "repoId": repo_id })),
        ("refs/list", json!({ "repoId": repo_id })),
        ("stash/list", json!({ "repoId": repo_id })),
        ("submodule/list", json!({ "repoId": repo_id })),
        ("graph/search", json!({ "repoId": repo_id, "query": "x" })),
        ("conflict/read", json!({ "repoId": repo_id, "path": "a" })),
    ] {
        let error = call(&engine, method, params).await.unwrap_err();
        assert_eq!(kind(&error), ErrorKind::RepoNotOpen, "{method}");
    }
}
