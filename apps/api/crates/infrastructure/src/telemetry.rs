//! Inicializacion de trazas y logs con `tracing`.

use std::io::IsTerminal;

use thiserror::Error;
use tracing_subscriber::EnvFilter;

use crate::config::TelemetryConfig;

/// Fallo al inicializar la telemetria.
#[derive(Debug, Error)]
pub enum TelemetryError {
    /// El filtro de nivel configurado no es valido.
    #[error("invalid log filter {filter:?}: {source}")]
    InvalidFilter {
        /// Filtro rechazado.
        filter: String,
        /// Error original del parser.
        #[source]
        source: tracing_subscriber::filter::ParseError,
    },
    /// Ya habia un subscriber global instalado.
    #[error("a global tracing subscriber was already installed")]
    AlreadyInitialized,
}

/// Instala el subscriber global de `tracing`.
///
/// Los mensajes del servicio van en ingles; cada linea lleva el target para
/// saber que crate la escribio. Los colores solo salen en una terminal: en
/// `docker logs` o en un fichero serian secuencias de escape sin sentido.
///
/// # Errors
///
/// Devuelve [`TelemetryError`] si el filtro no es valido o si ya habia un
/// subscriber instalado.
pub fn init_tracing(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    let filter = build_filter(&config.log_level)?;
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_ansi(std::io::stdout().is_terminal())
        .try_init()
        .map_err(|_| TelemetryError::AlreadyInitialized)
}

fn build_filter(level: &str) -> Result<EnvFilter, TelemetryError> {
    EnvFilter::try_new(level).map_err(|source| TelemetryError::InvalidFilter {
        filter: level.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests;
