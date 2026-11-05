//! Notificaciones del motor hacia el webview por `Channel` de Tauri.
//!
//! El webview se suscribe una vez con el comando `rpc_subscribe` y recibe por su
//! canal cada notificacion (`repo/changed`). Un canal que ya no se puede usar (la
//! ventana se recargo o se cerro) se descarta en el siguiente envio.

use std::sync::{Mutex, PoisonError};

use git_rudder_engine::NotificationSink;
use git_rudder_protocol::RpcNotification;
use tauri::ipc::Channel;

/// Canales abiertos por los webviews.
#[derive(Default)]
pub struct NotificationChannels {
    channels: Mutex<Vec<Channel<RpcNotification>>>,
}

impl std::fmt::Debug for NotificationChannels {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NotificationChannels")
            .field("open", &self.len())
            .finish()
    }
}

impl NotificationChannels {
    /// Anade un canal.
    pub fn subscribe(&self, channel: Channel<RpcNotification>) {
        self.lock().push(channel);
    }

    /// Canales abiertos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// No hay ningun canal.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Channel<RpcNotification>>> {
        self.channels.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl NotificationSink for NotificationChannels {
    fn notify(&self, notification: RpcNotification) {
        self.lock()
            .retain(|channel| channel.send(notification.clone()).is_ok());
    }
}

#[cfg(test)]
mod tests;
