//! Carga de la configuracion completa del servicio.

use std::net::SocketAddr;

use figment::Figment;
use figment::providers::{Env, Serialized};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::database_config::DatabaseConfig;
use super::server_config::ServerConfig;
use super::telemetry_config::TelemetryConfig;

/// Prefijo de todas las variables de entorno del servicio.
pub const ENV_PREFIX: &str = "RAMAL__";

/// Separador de niveles dentro del nombre de la variable.
const ENV_SEPARATOR: &str = "__";

/// Fallo al construir la configuracion.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// El entorno no encaja con el esquema: una clave desconocida, un tipo
    /// equivocado.
    ///
    /// El error de figment ocupa varios cientos de bytes, asi que va en un `Box`
    /// para que `Result<_, ConfigError>` siga siendo barato de mover.
    #[error("invalid configuration: {0}")]
    Invalid(#[source] Box<figment::Error>),
    /// La direccion de escucha no es valida.
    #[error("invalid listen address {host}:{port}")]
    InvalidAddress {
        /// Host configurado.
        host: String,
        /// Puerto configurado.
        port: u16,
    },
}

/// Configuracion completa del servicio.
///
/// Rechaza las claves que no conoce: una variable `RAMAL__` mal escrita hace
/// fallar el arranque en vez de ignorarse en silencio con su valor por defecto.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AppConfig {
    /// Servidor HTTP.
    pub server: ServerConfig,
    /// Base de datos.
    pub database: DatabaseConfig,
    /// Telemetria.
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
    /// Carga la configuracion desde las variables de entorno del proceso.
    ///
    /// # Errors
    ///
    /// Devuelve [`ConfigError::Invalid`] si alguna variable no encaja.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_provider(Env::prefixed(ENV_PREFIX).split(ENV_SEPARATOR))
    }

    /// Carga la configuracion desde un proveedor de figment; lo que falte cae en
    /// los valores por defecto.
    ///
    /// # Errors
    ///
    /// Devuelve [`ConfigError::Invalid`] si el proveedor no encaja.
    pub fn from_provider(provider: impl figment::Provider) -> Result<Self, ConfigError> {
        Figment::new()
            .merge(Serialized::defaults(Self::default()))
            .merge(provider)
            .extract()
            .map_err(|error| ConfigError::Invalid(Box::new(error)))
    }

    /// Direccion de escucha resuelta.
    ///
    /// # Errors
    ///
    /// Devuelve [`ConfigError::InvalidAddress`] si el host no es una IP.
    pub fn socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        self.server.socket_addr()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail loudly on purpose; a panic here is the assertion"
    )]
    #![allow(
        clippy::result_large_err,
        reason = "figment::Jail fixes the closure signature and its error type"
    )]

    use figment::Jail;

    use super::{AppConfig, ConfigError};

    /// `just` carga `.env` en el entorno de los tests: sin limpiar, una variable
    /// de la maquina cambiaria el resultado.
    fn load(jail: &mut Jail) -> Result<AppConfig, ConfigError> {
        jail.clear_env();
        AppConfig::from_env()
    }

    #[test]
    fn falls_back_to_defaults_when_the_environment_is_empty() {
        Jail::expect_with(|jail| {
            let config = load(jail).expect("defaults should load");
            assert_eq!(config, AppConfig::default());
            assert_eq!(config.server.port, 8080);
            assert_eq!(config.telemetry.log_level, "info");
            assert_eq!(config.database.max_connections, 5);
            Ok(())
        });
    }

    #[test]
    fn reads_nested_keys_from_double_underscore_variables() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("RAMAL__SERVER__HOST", "127.0.0.1");
            jail.set_env("RAMAL__SERVER__PORT", "9090");
            jail.set_env("RAMAL__DATABASE__URL", "postgres://u:p@db:5432/x");
            jail.set_env("RAMAL__DATABASE__MAX_CONNECTIONS", "9");
            jail.set_env("RAMAL__DATABASE__ACQUIRE_TIMEOUT_MS", "1500");
            jail.set_env("RAMAL__TELEMETRY__LOG_LEVEL", "debug");
            let config = AppConfig::from_env().expect("environment should load");
            assert_eq!(config.database.url, "postgres://u:p@db:5432/x");
            assert_eq!(config.database.max_connections, 9);
            assert_eq!(config.database.acquire_timeout_ms, 1500);
            assert_eq!(config.telemetry.log_level, "debug");
            assert_eq!(
                config.socket_addr().expect("address should resolve"),
                "127.0.0.1:9090".parse().expect("literal is valid")
            );
            Ok(())
        });
    }

    #[test]
    fn rejects_a_port_that_is_not_a_number() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("RAMAL__SERVER__PORT", "not-a-port");
            assert!(matches!(
                AppConfig::from_env(),
                Err(ConfigError::Invalid(_))
            ));
            Ok(())
        });
    }

    #[test]
    fn rejects_a_key_it_does_not_know() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("RAMAL__DATABASE__PASSWORD", "secret");
            let error = AppConfig::from_env().expect_err("unknown keys must fail");
            assert!(error.to_string().contains("invalid configuration"));
            Ok(())
        });
    }

    #[test]
    fn ignores_variables_without_the_double_underscore_prefix() {
        Jail::expect_with(|jail| {
            jail.clear_env();
            // Las de los tests de PostgreSQL llevan un solo guion bajo a proposito.
            jail.set_env("RAMAL_TEST_DATABASE_URL", "postgres://elsewhere");
            assert_eq!(
                AppConfig::from_env().expect("the variable is not ours"),
                AppConfig::default()
            );
            Ok(())
        });
    }
}
