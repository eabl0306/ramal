//! Senal de apagado ordenado.

use std::time::Duration;

/// Margen que se da a las peticiones en vuelo antes de cortar. Queda por debajo
/// de los 30 segundos que suele esperar un orquestador antes de matar el proceso.
pub const GRACE_PERIOD: Duration = Duration::from_secs(10);

/// Termina cuando llega SIGINT o SIGTERM.
pub async fn signal() {
    let reason = wait_for_signal().await;
    tracing::info!(signal = reason, "shutting down gracefully");
}

#[cfg(unix)]
async fn wait_for_signal() -> &'static str {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = match signal(SignalKind::terminate()) {
        Ok(stream) => stream,
        Err(error) => {
            tracing::error!(%error, "could not listen for SIGTERM");
            wait_for_ctrl_c().await;
            return "SIGINT";
        }
    };
    tokio::select! {
        () = wait_for_ctrl_c() => "SIGINT",
        _ = terminate.recv() => "SIGTERM",
    }
}

#[cfg(not(unix))]
async fn wait_for_signal() -> &'static str {
    wait_for_ctrl_c().await;
    "SIGINT"
}

/// Si ni siquiera Ctrl+C se puede escuchar, se espera para siempre: el proceso
/// sigue atendiendo y lo parara quien lo lanzo.
async fn wait_for_ctrl_c() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "could not listen for SIGINT");
        std::future::pending::<()>().await;
    }
}
