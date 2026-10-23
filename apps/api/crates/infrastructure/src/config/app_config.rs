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
pub const ENV_PREFIX: &str = "GIT_RUDDER__";

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
/// Rechaza las claves que no conoce: una variable `GIT_RUDDER__` mal escrita hace
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
mod tests;
