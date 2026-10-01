//! `graph/window` de punta a punta contra repositorios reales de prueba.

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
use pretty_assertions::assert_eq;
use ramal_engine::{CallContext, Engine, EngineBuilder, EngineConfig};
use ramal_protocol::graph::{EdgeKind, GraphWindow, RefKind};
use ramal_protocol::{ErrorKind, RpcError, error_code};
use serde_json::{Value, json};

async fn call(engine: &Engine, method: &str, params: Value) -> Result<Value, RpcError> {
    engine
        .dispatch(method, params, CallContext::webview())
        .await
}

/// Motor con el repositorio abierto por el host; devuelve el motor y el repoId.
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
    let repo_id = handle["repoId"].as_str().unwrap().to_owned();
    (engine, repo_id)
}

async fn window(engine: &Engine, repo_id: &str, offset: u32, limit: u32) -> GraphWindow {
    let params = json!({ "repoId": repo_id, "offset": offset, "limit": limit });
    serde_json::from_value(call(engine, "graph/window", params).await.unwrap()).unwrap()
}

#[tokio::test]
async fn the_graph_of_a_merged_feature_has_lanes_edges_and_refs() {
    let repo = FixtureRepo::with_merged_feature();
    let (engine, repo_id) = opened(&repo).await;

    let graph = window(&engine, &repo_id, 0, 200).await;

    assert_eq!(graph.repo_id, repo_id);
    assert_eq!(graph.offset, 0);
    assert!(!graph.has_more);
    assert_eq!(graph.lane_count, 2);
    let summaries: Vec<&str> = graph.rows.iter().map(|row| row.summary.as_str()).collect();
    assert_eq!(
        summaries,
        vec![
            "Merge branch 'feature'",
            "Add graph",
            "Fix login",
            "Initial commit"
        ]
    );
    let lanes: Vec<u32> = graph.rows.iter().map(|row| row.lane).collect();
    assert_eq!(lanes, vec![0, 1, 0, 0]);

    let merge = &graph.rows[0];
    assert!(merge.is_head);
    assert_eq!(merge.sha, repo.rev("main"));
    assert_eq!(merge.parents, vec![repo.rev("main^1"), repo.rev("main^2")]);
    assert_eq!(merge.edges[1].kind, EdgeKind::Merge);
    assert_eq!(merge.edges[1].parent_row, Some(1));
    assert_eq!(
        (
            merge.edges[1].from_lane,
            merge.edges[1].travel_lane,
            merge.edges[1].to_lane
        ),
        (0, 1, 1)
    );
    let labels: Vec<(&str, RefKind, bool)> = merge
        .refs
        .iter()
        .map(|label| (label.name.as_str(), label.kind, label.is_head))
        .collect();
    assert_eq!(
        labels,
        vec![
            ("main", RefKind::LocalBranch, true),
            ("v1.0", RefKind::Tag, false)
        ]
    );
    assert_eq!(merge.authored_at, "2026-09-30T10:04:00+02:00");
    assert_eq!(merge.author_name, "Ana Ruiz");
}

#[tokio::test]
async fn windows_keep_absolute_rows_and_say_when_there_is_more() {
    let repo = FixtureRepo::new();
    for index in 0..7 {
        repo.commit(&format!("commit {index}"));
    }
    let (engine, repo_id) = opened(&repo).await;

    let middle = window(&engine, &repo_id, 2, 3).await;
    assert_eq!(middle.rows.len(), 3);
    assert!(middle.has_more);
    assert_eq!(middle.rows[0].summary, "commit 4");
    assert_eq!(middle.rows[0].edges[0].parent_row, Some(3));

    let last = window(&engine, &repo_id, 5, 3).await;
    assert_eq!(last.rows.len(), 2);
    assert!(!last.has_more);
    assert!(last.rows[1].edges.is_empty());
}

#[tokio::test]
async fn an_edge_to_a_parent_past_the_loaded_commits_has_no_row() {
    let repo = FixtureRepo::new();
    for index in 0..5 {
        repo.commit(&format!("commit {index}"));
    }
    let (engine, repo_id) = opened(&repo).await;

    let first = window(&engine, &repo_id, 0, 1).await;

    assert_eq!(first.rows[0].edges[0].parent_row, Some(1));
    let value = call(
        &engine,
        "graph/window",
        json!({ "repoId": repo_id, "offset": 0, "limit": 1 }),
    )
    .await
    .unwrap();
    assert!(value["rows"][0]["edges"][0].get("parentRow").is_some());
}

#[tokio::test]
async fn the_json_of_a_row_uses_camel_case() {
    let repo = FixtureRepo::with_merged_feature();
    let (engine, repo_id) = opened(&repo).await;
    let params = json!({ "repoId": repo_id, "offset": 0, "limit": 1 });

    let value = call(&engine, "graph/window", params).await.unwrap();

    let row = &value["rows"][0];
    for key in [
        "sha",
        "shortSha",
        "parents",
        "lane",
        "edges",
        "summary",
        "authorName",
        "authorEmail",
        "authoredAt",
        "refs",
        "isHead",
    ] {
        assert!(row.get(key).is_some(), "missing {key}");
    }
    assert_eq!(row["edges"][0]["kind"], json!("firstParent"));
    assert_eq!(row["refs"][0]["fullName"], json!("refs/heads/main"));
    assert_eq!(value["hasMore"], json!(true));
    assert_eq!(value["laneCount"], json!(2));
}

#[tokio::test]
async fn an_empty_repository_has_an_empty_graph() {
    let repo = FixtureRepo::new();
    let (engine, repo_id) = opened(&repo).await;

    let graph = window(&engine, &repo_id, 0, 200).await;

    assert!(graph.rows.is_empty());
    assert!(!graph.has_more);
}

#[tokio::test]
async fn an_unknown_repo_id_is_not_open() {
    let repo = FixtureRepo::with_merged_feature();
    let (engine, _) = opened(&repo).await;
    let params = json!({ "repoId": "repo-99", "offset": 0, "limit": 10 });

    let error = call(&engine, "graph/window", params).await.unwrap_err();

    assert_eq!(error.data.unwrap().kind, ErrorKind::RepoNotOpen);
    assert_eq!(error.message, "repository repo-99 is not open");
}

/// Pagina siguiente con la huella de la anterior, como la pide la interfaz.
async fn next_page(
    engine: &Engine,
    repo_id: &str,
    previous: &GraphWindow,
    limit: u32,
) -> Result<GraphWindow, RpcError> {
    let offset = previous.offset + u32::try_from(previous.rows.len()).unwrap();
    let params = json!({
        "repoId": repo_id, "offset": offset, "limit": limit, "snapshot": previous.snapshot
    });
    call(engine, "graph/window", params)
        .await
        .map(|value| serde_json::from_value(value).unwrap())
}

#[tokio::test]
async fn pages_of_an_unchanged_repository_share_the_snapshot() {
    let repo = FixtureRepo::new();
    for index in 1..=6 {
        repo.commit(&format!("c{index}"));
    }
    let (engine, repo_id) = opened(&repo).await;

    let first = window(&engine, &repo_id, 0, 3).await;
    let second = next_page(&engine, &repo_id, &first, 3).await.unwrap();

    assert_eq!(first.snapshot.len(), 32);
    assert_eq!(second.snapshot, first.snapshot);
    let summaries: Vec<&str> = first
        .rows
        .iter()
        .chain(&second.rows)
        .map(|row| row.summary.as_str())
        .collect();
    assert_eq!(summaries, vec!["c6", "c5", "c4", "c3", "c2", "c1"]);
}

#[tokio::test]
async fn a_commit_between_two_pages_makes_the_next_page_stale() {
    // El fallo real: c1..c6, pagina (0, 3), commit desde la terminal, pagina
    // (3, 3). Se unian ['c6', 'c5', 'c4', 'c4', 'c3', 'c2'], con c4 repetido.
    let repo = FixtureRepo::new();
    for index in 1..=6 {
        repo.commit(&format!("c{index}"));
    }
    let (engine, repo_id) = opened(&repo).await;
    let first = window(&engine, &repo_id, 0, 3).await;

    repo.commit("c7");
    let error = next_page(&engine, &repo_id, &first, 3).await.unwrap_err();

    assert_eq!(error.data.unwrap().kind, ErrorKind::StaleWindow);
    assert_eq!(error.code, error_code::ENGINE_ERROR);
    let again = window(&engine, &repo_id, 0, 3).await;
    let rest = next_page(&engine, &repo_id, &again, 10).await.unwrap();
    assert_ne!(again.snapshot, first.snapshot);
    let summaries: Vec<&str> = again
        .rows
        .iter()
        .chain(&rest.rows)
        .map(|row| row.summary.as_str())
        .collect();
    assert_eq!(summaries, vec!["c7", "c6", "c5", "c4", "c3", "c2", "c1"]);
}

#[tokio::test]
async fn checking_out_another_branch_also_makes_the_next_page_stale() {
    let repo = FixtureRepo::with_merged_feature();
    let (engine, repo_id) = opened(&repo).await;
    let first = window(&engine, &repo_id, 0, 1).await;

    repo.checkout("feature");
    let error = next_page(&engine, &repo_id, &first, 1).await.unwrap_err();

    assert_eq!(error.data.unwrap().kind, ErrorKind::StaleWindow);
}

#[tokio::test]
async fn a_limit_outside_one_to_a_thousand_is_invalid() {
    let repo = FixtureRepo::with_merged_feature();
    let (engine, repo_id) = opened(&repo).await;

    for limit in [0, 1001] {
        let params = json!({ "repoId": repo_id, "offset": 0, "limit": limit });
        let error = call(&engine, "graph/window", params).await.unwrap_err();
        assert_eq!(error.data.unwrap().kind, ErrorKind::InvalidParams);
        assert_eq!(error.code, error_code::INVALID_PARAMS);
    }
    let negative = json!({ "repoId": repo_id, "offset": -1, "limit": 10 });
    let error = call(&engine, "graph/window", negative).await.unwrap_err();
    assert_eq!(error.data.unwrap().kind, ErrorKind::InvalidParams);
}
