use std::sync::{Arc, Mutex, PoisonError};

use git_rudder_engine::NotificationSink as _;
use git_rudder_protocol::{JsonRpcVersion, RpcNotification};
use serde_json::json;
use tauri::ipc::{Channel, InvokeResponseBody};

use super::NotificationChannels;

fn notification() -> RpcNotification {
    RpcNotification {
        jsonrpc: JsonRpcVersion::V2,
        method: "repo/changed".to_owned(),
        params: json!({ "repoId": "repo-1" }),
    }
}

#[test]
fn every_open_channel_receives_the_notification() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&received);
    let channel = Channel::new(move |body: InvokeResponseBody| {
        if let InvokeResponseBody::Json(text) = body {
            sink.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(text);
        }
        Ok(())
    });
    let channels = NotificationChannels::default();
    assert!(channels.is_empty());
    channels.subscribe(channel);

    channels.notify(notification());

    let received = received
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    assert_eq!(received.len(), 1);
    assert!(
        received
            .first()
            .is_some_and(|text| text.contains("repo/changed"))
    );
    assert!(format!("{channels:?}").contains("open: 1"));
}

#[test]
fn a_broken_channel_is_dropped() {
    let channels = NotificationChannels::default();
    channels.subscribe(Channel::new(|_| Err(tauri::Error::FailedToReceiveMessage)));

    channels.notify(notification());

    assert_eq!(channels.len(), 0);
}
