//! Tabla de metodos y forma de los parametros y resultados.

#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and this test does not use every dependency"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use git_rudder_protocol::engine::{EngineInfo, EngineInfoParams, GitInfo};
use git_rudder_protocol::graph::{EdgeKind, GraphEdge, GraphWindowParams, RefKind, RefLabel};
use git_rudder_protocol::methods::{GraphWindowMethod, RepoOpenMethod};
use git_rudder_protocol::repo::{OpenRepoParams, RepoHandle};
use git_rudder_protocol::repos::TrustFinding;
use git_rudder_protocol::{METHODS, MethodInfo, RpcMethod};
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn every_method_with_paths_or_trust_decisions_is_host_only() {
    let host_only: Vec<&str> = METHODS
        .iter()
        .filter(|method| method.host_only)
        .map(|method| method.name)
        .collect();

    assert_eq!(
        host_only,
        vec![
            "git/configure",
            "repo/open",
            "repo/trust",
            "repos/summary",
            "repos/scan",
            "submodule/open"
        ]
    );
    assert_eq!(METHODS.len(), 20);
    assert_eq!(
        MethodInfo::of::<RepoOpenMethod>(),
        MethodInfo {
            name: "repo/open",
            host_only: true
        }
    );
    assert_eq!(GraphWindowMethod::NAME, "graph/window");
}

#[test]
fn methods_are_found_by_name() {
    assert_eq!(
        MethodInfo::find("graph/window"),
        Some(MethodInfo {
            name: "graph/window",
            host_only: false
        })
    );
    assert_eq!(MethodInfo::find("repo/delete"), None);
}

#[test]
fn params_reject_unknown_fields() {
    let extra = json!({ "repoId": "repo-1", "offset": 0, "limit": 10, "path": "/etc" });
    assert!(serde_json::from_value::<GraphWindowParams>(extra).is_err());
    assert!(serde_json::from_value::<EngineInfoParams>(json!({ "x": 1 })).is_err());
    assert!(
        serde_json::from_value::<OpenRepoParams>(json!({ "path": "/w", "repoId": "r" })).is_err()
    );
}

#[test]
fn params_use_camel_case() {
    let params: GraphWindowParams =
        serde_json::from_value(json!({ "repoId": "repo-1", "offset": 200, "limit": 100 })).unwrap();

    assert_eq!(params.repo_id, "repo-1");
    assert_eq!(params.offset, 200);
    assert_eq!(params.snapshot, None);
    assert!(serde_json::from_value::<EngineInfoParams>(json!({})).is_ok());
}

#[test]
fn the_window_snapshot_is_optional_in_params_and_omitted_when_missing() {
    let next: GraphWindowParams = serde_json::from_value(
        json!({ "repoId": "repo-1", "offset": 200, "limit": 100, "snapshot": "ab12" }),
    )
    .unwrap();
    assert_eq!(next.snapshot.as_deref(), Some("ab12"));

    let first = GraphWindowParams {
        snapshot: None,
        ..next
    };
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        json!({ "repoId": "repo-1", "offset": 200, "limit": 100 })
    );
}

#[test]
fn a_repo_handle_omits_missing_head_fields() {
    let detached = RepoHandle {
        repo_id: "repo-1".to_owned(),
        root: "/work/git-rudder".to_owned(),
        head_ref: None,
        head_sha: Some("a".repeat(40)),
        is_detached: true,
        is_empty: false,
        trusted: false,
        trust_findings: vec![TrustFinding::Hook {
            name: "post-checkout".to_owned(),
        }],
    };

    assert_eq!(
        serde_json::to_value(&detached).unwrap(),
        json!({
            "repoId": "repo-1", "root": "/work/git-rudder", "headSha": "a".repeat(40),
            "isDetached": true, "isEmpty": false, "trusted": false,
            "trustFindings": [{ "kind": "hook", "name": "post-checkout" }]
        })
    );
}

#[test]
fn a_graph_edge_writes_a_null_parent_row_and_camel_case_kinds() {
    let edge = GraphEdge {
        parent_sha: "b".repeat(40),
        parent_row: None,
        from_lane: 0,
        travel_lane: 2,
        to_lane: 2,
        kind: EdgeKind::FirstParent,
    };
    let label = RefLabel {
        name: "origin/main".to_owned(),
        full_name: "refs/remotes/origin/main".to_owned(),
        kind: RefKind::RemoteBranch,
        is_head: false,
    };

    assert_eq!(
        serde_json::to_value(&edge).unwrap(),
        json!({
            "parentSha": "b".repeat(40), "parentRow": null, "fromLane": 0,
            "travelLane": 2, "toLane": 2, "kind": "firstParent"
        })
    );
    assert_eq!(
        serde_json::to_value(&label).unwrap().get("kind"),
        Some(&json!("remoteBranch"))
    );
}

#[test]
fn engine_info_reports_versions_in_camel_case() {
    let info = EngineInfo {
        engine_version: "0.1.0".to_owned(),
        protocol_version: 1,
        git: GitInfo {
            path: "/usr/bin/git".to_owned(),
            version: "2.45.1".to_owned(),
            supported: true,
        },
    };

    assert_eq!(
        serde_json::to_value(&info).unwrap(),
        json!({
            "engineVersion": "0.1.0", "protocolVersion": 1,
            "git": { "path": "/usr/bin/git", "version": "2.45.1", "supported": true }
        })
    );
}

#[test]
fn tagged_enums_carry_their_kind_and_camel_case_fields() {
    use git_rudder_protocol::diff::{DiffFileParams, DiffTarget};
    use git_rudder_protocol::status::Operation;

    let params: DiffFileParams = serde_json::from_value(json!({
        "repoId": "repo-1", "path": "a.txt",
        "target": { "kind": "range", "from": "a", "to": "b" },
        "options": { "whitespace": "ignoreAtEol", "wordDiff": true }
    }))
    .unwrap();
    assert_eq!(
        params.target,
        DiffTarget::Range {
            from: "a".to_owned(),
            to: "b".to_owned()
        }
    );

    let rebase = Operation::Rebase {
        branch: Some("main".to_owned()),
        onto: None,
        step: Some(3),
        total: Some(7),
        stopped_at: Some("c".repeat(40)),
        interactive: true,
    };
    assert_eq!(
        serde_json::to_value(&rebase).unwrap(),
        json!({
            "kind": "rebase", "branch": "main", "step": 3, "total": 7,
            "stoppedAt": "c".repeat(40), "interactive": true
        })
    );
}

#[test]
fn a_graph_filter_rejects_unknown_fields_and_is_optional() {
    use git_rudder_protocol::graph::GraphFilter;

    let params: GraphWindowParams = serde_json::from_value(json!({
        "repoId": "repo-1", "offset": 0, "limit": 10,
        "filter": { "only": ["refs/heads/main"], "since": "2026-01-01" }
    }))
    .unwrap();
    assert_eq!(
        params.filter,
        Some(GraphFilter {
            only: Some(vec!["refs/heads/main".to_owned()]),
            since: Some("2026-01-01".to_owned()),
            ..GraphFilter::default()
        })
    );
    assert!(serde_json::from_value::<GraphFilter>(json!({ "branch": "x" })).is_err());
}
