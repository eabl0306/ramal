#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;
use serde_json::json;

use super::{
    ErrorKind, JsonRpcVersion, RequestId, RpcError, RpcNotification, RpcOutcome, RpcRequest,
    RpcResponse, error_code,
};

#[test]
fn a_request_reads_and_writes_the_json_rpc_shape() {
    let raw = json!({
        "jsonrpc": "2.0", "id": 7, "method": "graph/window",
        "params": { "repoId": "repo-1", "offset": 0, "limit": 200 }
    });

    let request: RpcRequest = serde_json::from_value(raw.clone()).unwrap();

    assert_eq!(request.id, RequestId::new(7));
    assert_eq!(request.method, "graph/window");
    assert_eq!(serde_json::to_value(&request).unwrap(), raw);
}

#[test]
fn a_request_without_id_is_a_notification_and_params_default_to_an_object() {
    let request: RpcRequest =
        serde_json::from_value(json!({ "jsonrpc": "2.0", "method": "$/cancel" })).unwrap();

    assert_eq!(request.id, None);
    assert_eq!(request.params, json!({}));
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        json!({ "jsonrpc": "2.0", "method": "$/cancel", "params": {} })
    );
}

#[test]
fn request_ids_must_be_safe_javascript_numbers() {
    let too_big = json!({ "jsonrpc": "2.0", "id": 2_147_483_648_u64, "method": "engine/info" });
    let text = json!({ "jsonrpc": "2.0", "id": "7", "method": "engine/info" });
    let negative = json!({ "jsonrpc": "2.0", "id": -1, "method": "engine/info" });

    for raw in [too_big, text, negative] {
        assert!(serde_json::from_value::<RpcRequest>(raw).is_err());
    }
    assert_eq!(RequestId::new(RequestId::MAX).unwrap().get(), 2_147_483_647);
    assert_eq!(RequestId::new(RequestId::MAX + 1), None);
}

#[test]
fn only_version_two_is_accepted() {
    let raw = json!({ "jsonrpc": "1.0", "id": 1, "method": "engine/info" });

    assert!(serde_json::from_value::<RpcRequest>(raw).is_err());
    assert_eq!(JsonRpcVersion::default(), JsonRpcVersion::V2);
}

#[test]
fn a_success_response_carries_result_and_no_error() {
    let response = RpcResponse::success(RequestId::new(3), json!({ "ok": true }));

    assert_eq!(
        serde_json::to_value(&response).unwrap(),
        json!({ "jsonrpc": "2.0", "id": 3, "result": { "ok": true } })
    );
}

#[test]
fn an_error_response_carries_the_kind_and_a_null_id_when_unknown() {
    let error = RpcError::new(
        error_code::ENGINE_ERROR,
        "repository repo-9 is not open",
        ErrorKind::RepoNotOpen,
    );
    let response = RpcResponse::failure(None, error);

    assert_eq!(
        serde_json::to_value(&response).unwrap(),
        json!({
            "jsonrpc": "2.0", "id": null,
            "error": {
                "code": -32000, "message": "repository repo-9 is not open",
                "data": { "kind": "repo_not_open" }
            }
        })
    );
}

#[test]
fn a_response_round_trips_through_json() {
    let raw = json!({
        "jsonrpc": "2.0", "id": 9,
        "error": { "code": -32602, "message": "bad", "data": { "kind": "invalid_params" } }
    });

    let response: RpcResponse = serde_json::from_value(raw).unwrap();

    assert_eq!(response.id, RequestId::new(9));
    let RpcOutcome::Error(error) = response.outcome else {
        unreachable!("expected an error outcome");
    };
    assert_eq!(error.code, error_code::INVALID_PARAMS);
    assert_eq!(error.data.unwrap().kind, ErrorKind::InvalidParams);
}

#[test]
fn error_kinds_are_snake_case_literals() {
    let kinds = [
        (ErrorKind::InvalidParams, "invalid_params"),
        (ErrorKind::RepoNotOpen, "repo_not_open"),
        (ErrorKind::NotARepository, "not_a_repository"),
        (ErrorKind::GitNotFound, "git_not_found"),
        (ErrorKind::GitVersionUnsupported, "git_version_unsupported"),
        (ErrorKind::GitFailed, "git_failed"),
        (ErrorKind::StaleWindow, "stale_window"),
        (ErrorKind::Internal, "internal"),
    ];
    for (kind, text) in kinds {
        assert_eq!(serde_json::to_value(kind).unwrap(), json!(text));
    }
}

#[test]
fn an_error_without_data_omits_the_field() {
    let error = RpcError {
        code: error_code::INTERNAL_ERROR,
        message: "boom".to_owned(),
        data: None,
    };

    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        json!({ "code": -32603, "message": "boom" })
    );
}

#[test]
fn a_notification_has_no_id() {
    let notification = RpcNotification {
        jsonrpc: JsonRpcVersion::V2,
        method: "repo/changed".to_owned(),
        params: json!({ "repoId": "repo-1" }),
    };

    assert_eq!(
        serde_json::to_value(&notification).unwrap(),
        json!({ "jsonrpc": "2.0", "method": "repo/changed", "params": { "repoId": "repo-1" } })
    );
}
