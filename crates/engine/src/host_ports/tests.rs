use std::sync::{Arc, Mutex, PoisonError};

use git_rudder_core_application::{EngineEvent, EventSink as _};
use git_rudder_core_domain::RepoId;
use git_rudder_protocol::RpcNotification;
use serde_json::json;

use super::{NotificationSink, ProtocolEvents};

#[derive(Default)]
struct Recorder(Mutex<Vec<RpcNotification>>);

impl NotificationSink for Recorder {
    fn notify(&self, notification: RpcNotification) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(notification);
    }
}

#[test]
fn a_repo_change_becomes_a_repo_changed_notification_without_id() {
    let recorder = Arc::new(Recorder::default());
    let events = ProtocolEvents::new(recorder.clone());

    events.publish(EngineEvent::RepoChanged {
        repo_id: RepoId::new("repo-0123456789abcdef"),
    });

    let sent = recorder
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let [notification] = sent.as_slice() else {
        unreachable!("one notification: {sent:?}")
    };
    assert_eq!(notification.method, "repo/changed");
    assert_eq!(
        notification.params,
        json!({ "repoId": "repo-0123456789abcdef" })
    );
    let wire = serde_json::to_value(notification).unwrap_or_default();
    assert_eq!(wire.get("id"), None, "notifications travel without id");
}
