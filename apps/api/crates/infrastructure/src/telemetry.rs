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
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail loudly on purpose; a panic here is the assertion"
    )]

    use super::{TelemetryError, build_filter, init_tracing};
    use crate::config::TelemetryConfig;

    #[test]
    fn accepts_a_plain_level() {
        assert!(build_filter("info").is_ok());
    }

    #[test]
    fn accepts_a_per_target_directive() {
        assert!(build_filter("warn,ramal_api_http=debug").is_ok());
    }

    #[test]
    fn rejects_a_malformed_directive() {
        let error = build_filter("ramal_api_http=not-a-level").expect_err("must fail");
        assert!(error.to_string().contains("invalid log filter"));
    }

    #[test]
    fn a_second_installation_is_reported_instead_of_panicking() {
        let config = TelemetryConfig::default();
        // El primero puede ganar o perder la carrera con otro test del binario;
        // el segundo siempre encuentra uno instalado.
        let _ = init_tracing(&config);
        assert!(matches!(
            init_tracing(&config),
            Err(TelemetryError::AlreadyInitialized)
        ));
    }

    #[test]
    fn an_invalid_level_never_installs_anything() {
        let config = TelemetryConfig {
            log_level: "=".to_owned(),
        };
        assert!(matches!(
            init_tracing(&config),
            Err(TelemetryError::InvalidFilter { .. })
        ));
    }
}
