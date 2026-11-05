//! Lo que los hosts implementan para el motor.
//!
//! Los hosts solo dependen del motor y del protocolo, asi que los puertos que les
//! tocan se expresan aqui con tipos del protocolo: el escritorio entrega cada
//! notificacion por un `Channel` de Tauri y el sidecar la escribe como un marco
//! en stdout. El motor traduce sus eventos de dominio a esas notificaciones.

use std::sync::Arc;

use git_rudder_core_application::{EngineEvent, EventSink};
use git_rudder_protocol::notifications::{
    EngineNotification as _, RepoChangedNotification, RepoChangedParams,
};
use git_rudder_protocol::{JsonRpcVersion, RpcNotification};

/// Donde el host recibe las notificaciones del motor.
pub trait NotificationSink: Send + Sync {
    /// Entrega una notificacion. No puede fallar: si nadie escucha, se pierde.
    fn notify(&self, notification: RpcNotification);
}

/// El `EventSink` del nucleo sobre el `NotificationSink` del host.
pub(crate) struct ProtocolEvents {
    sink: Arc<dyn NotificationSink>,
}

impl ProtocolEvents {
    pub(crate) fn new(sink: Arc<dyn NotificationSink>) -> Self {
        Self { sink }
    }
}

impl EventSink for ProtocolEvents {
    fn publish(&self, event: EngineEvent) {
        let notification = match event {
            EngineEvent::RepoChanged { repo_id } => notification(
                RepoChangedNotification::NAME,
                &RepoChangedParams {
                    repo_id: repo_id.as_str().to_owned(),
                },
            ),
        };
        if let Some(notification) = notification {
            self.sink.notify(notification);
        }
    }
}

fn notification(method: &str, params: &impl serde::Serialize) -> Option<RpcNotification> {
    serde_json::to_value(params)
        .map(|params| RpcNotification {
            jsonrpc: JsonRpcVersion::V2,
            method: method.to_owned(),
            params,
        })
        .inspect_err(|error| tracing::error!(%error, "could not serialize a notification"))
        .ok()
}

#[cfg(test)]
mod tests;
