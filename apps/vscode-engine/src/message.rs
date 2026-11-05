//! De los bytes de un marco a la respuesta del motor.
//!
//! El extension host ya valido los mensajes de los webviews, pero el sidecar no se
//! fia: JSON roto da -32700, un sobre invalido (id de texto o fuera de rango, sin
//! metodo, un array) da -32600, y en los dos casos el id es `null` si no se pudo
//! leer.

use git_rudder_engine::{Engine, NotificationSink};
use git_rudder_protocol::{
    ErrorKind, RequestId, RpcError, RpcNotification, RpcRequest, RpcResponse, error_code,
};
use serde_json::Value;

use crate::server::{MessageHandler, OutboxSender};

/// El motor como manejador de mensajes del servidor.
#[derive(Debug)]
pub struct EngineHandler {
    engine: Engine,
}

impl EngineHandler {
    /// Envuelve el motor.
    #[must_use]
    pub const fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

impl MessageHandler for EngineHandler {
    async fn handle(&self, message: Vec<u8>) -> Option<Vec<u8>> {
        let response = respond(&self.engine, &message).await?;
        match serde_json::to_vec(&response) {
            Ok(bytes) => Some(bytes),
            Err(error) => {
                tracing::error!(%error, "could not serialize a response");
                None
            }
        }
    }
}

/// Respuesta a un mensaje, o `None` si era una notificacion.
pub async fn respond(engine: &Engine, message: &[u8]) -> Option<RpcResponse> {
    let value: Value = match serde_json::from_slice(message) {
        Ok(value) => value,
        Err(error) => {
            return Some(failure(
                None,
                error_code::PARSE_ERROR,
                &format!("parse error: {error}"),
            ));
        }
    };
    let id = value
        .get("id")
        .and_then(|id| serde_json::from_value::<RequestId>(id.clone()).ok());
    match serde_json::from_value::<RpcRequest>(value) {
        Ok(request) => engine.handle(request).await,
        Err(error) => Some(failure(
            id,
            error_code::INVALID_REQUEST,
            &format!("invalid request: {error}"),
        )),
    }
}

/// Un sobre roto es un fallo de quien lo envia, no algo que el usuario arregle.
fn failure(id: Option<RequestId>, code: i32, message: &str) -> RpcResponse {
    tracing::warn!(code, message, "rejected a message");
    RpcResponse::failure(id, RpcError::new(code, message, ErrorKind::Internal))
}

/// Las notificaciones del motor como marcos en la cola de salida del servidor.
#[derive(Debug, Clone)]
pub struct FrameSink(OutboxSender);

impl FrameSink {
    /// Sink que escribe en `outbox`.
    #[must_use]
    pub const fn new(outbox: OutboxSender) -> Self {
        Self(outbox)
    }
}

impl NotificationSink for FrameSink {
    fn notify(&self, notification: RpcNotification) {
        match serde_json::to_vec(&notification) {
            Ok(bytes) => {
                // Si la salida ya se cerro no hay nadie a quien avisar.
                let _ = self.0.send(bytes);
            }
            Err(error) => tracing::error!(%error, "could not serialize a notification"),
        }
    }
}

#[cfg(test)]
mod tests;
