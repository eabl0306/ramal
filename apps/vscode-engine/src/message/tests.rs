#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

use pretty_assertions::assert_eq;
use ramal_engine::{EngineBuilder, EngineConfig};
use ramal_protocol::{RequestId, RpcOutcome, RpcResponse, error_code};
use serde_json::{Value, json};

use super::{EngineHandler, respond};
use crate::server::MessageHandler;

fn handler() -> EngineHandler {
    EngineHandler::new(EngineBuilder::new(EngineConfig::default()).build())
}

async fn answer(message: &[u8]) -> Option<RpcResponse> {
    let engine = EngineBuilder::new(EngineConfig::default()).build();
    respond(&engine, message).await
}

fn error_code_of(response: &RpcResponse) -> i32 {
    match &response.outcome {
        RpcOutcome::Error(error) => error.code,
        RpcOutcome::Result(result) => unreachable!("expected an error, got {result}"),
    }
}

#[tokio::test]
async fn broken_json_is_a_parse_error_with_a_null_id() {
    let response = answer(b"{\"jsonrpc\": \"2.0\", \"id\": 1,").await.unwrap();

    assert_eq!(response.id, None);
    assert_eq!(error_code_of(&response), error_code::PARSE_ERROR);
}

#[tokio::test]
async fn an_invalid_envelope_keeps_the_id_when_it_can_be_read() {
    let without_method = answer(br#"{"jsonrpc":"2.0","id":5}"#).await.unwrap();
    assert_eq!(without_method.id.map(RequestId::get), Some(5));
    assert_eq!(error_code_of(&without_method), error_code::INVALID_REQUEST);

    for raw in [
        br#"{"jsonrpc":"2.0","id":"5","method":"engine/info"}"#.as_slice(),
        br#"{"jsonrpc":"2.0","id":4294967295,"method":"engine/info"}"#,
        br#"[{"jsonrpc":"2.0","id":1,"method":"engine/info"}]"#,
        br#"{"jsonrpc":"1.0","id":1,"method":"engine/info"}"#,
    ] {
        let response = answer(raw).await.unwrap();
        assert_eq!(error_code_of(&response), error_code::INVALID_REQUEST);
    }
}

#[tokio::test]
async fn a_notification_has_no_answer() {
    let message = br#"{"jsonrpc":"2.0","method":"$/cancel","params":{"id":1}}"#;

    assert_eq!(handler().handle(message.to_vec()).await, None);
}

#[tokio::test]
async fn a_request_is_answered_with_serialized_json() {
    let message = br#"{"jsonrpc":"2.0","id":9,"method":"engine/info","params":{}}"#;

    let bytes = handler().handle(message.to_vec()).await.unwrap();

    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["id"], json!(9));
    assert_eq!(value["jsonrpc"], json!("2.0"));
    assert!(value.get("result").is_some() || value.get("error").is_some());
}
