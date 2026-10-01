//! Seccion `server`: donde escucha la API.

use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::app_config::ConfigError;

/// Configuracion del servidor HTTP (`RAMAL__SERVER__*`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerConfig {
    /// IP de escucha. `0.0.0.0` dentro de un contenedor.
    pub host: String,
    /// Puerto de escucha.
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_owned(),
            port: 8080,
        }
    }
}

impl ServerConfig {
    /// Direccion de escucha resuelta.
    ///
    /// # Errors
    ///
    /// Devuelve [`ConfigError::InvalidAddress`] si el host no es una IP: un
    /// nombre de maquina obligaria a resolver DNS antes de escuchar.
    pub fn socket_addr(&self) -> Result<SocketAddr, ConfigError> {
        self.host.parse::<IpAddr>().map_or_else(
            |_| {
                Err(ConfigError::InvalidAddress {
                    host: self.host.clone(),
                    port: self.port,
                })
            },
            |ip| Ok(SocketAddr::new(ip, self.port)),
        )
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

    use super::{ConfigError, ServerConfig};

    #[test]
    fn the_default_listens_on_every_interface() {
        let address = ServerConfig::default()
            .socket_addr()
            .expect("the default should resolve");
        assert_eq!(address.to_string(), "0.0.0.0:8080");
    }

    #[test]
    fn rejects_a_host_that_is_not_an_ip() {
        let server = ServerConfig {
            host: "not an ip".to_owned(),
            port: 8080,
        };
        let error = server.socket_addr().expect_err("a name is not an ip");
        assert!(matches!(error, ConfigError::InvalidAddress { .. }));
        assert_eq!(error.to_string(), "invalid listen address not an ip:8080");
    }
}
