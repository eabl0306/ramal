//! Seccion `telemetry`: que se registra en el log.

use serde::{Deserialize, Serialize};

/// Configuracion de telemetria (`GIT_RUDDER__TELEMETRY__*`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TelemetryConfig {
    /// Filtro de nivel con la sintaxis de `RUST_LOG`, por ejemplo
    /// `info,git_rudder_api_http=debug`.
    pub log_level: String,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_owned(),
        }
    }
}
