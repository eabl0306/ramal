//! Seccion `server`: donde escucha la API y que sirve ademas de las rutas.

use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::app_config::ConfigError;

/// Configuracion del servidor HTTP (`RAMAL__SERVER__*`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerConfig {
    /// IP de escucha. Por defecto solo loopback: `just api-run` en una red
    /// compartida no tiene por que abrir la API a toda la red. Los
    /// contenedores ponen `0.0.0.0` por entorno (compose.yaml y los Dockerfile).
    pub host: String,
    /// Puerto de escucha.
    pub port: u16,
    /// Sirve el visor Swagger UI en `/swagger-ui/`. Apagado por defecto: Swagger
    /// UI ha tenido fallos de XSS, y en el dominio de produccion correria con las
    /// cookies de sesion y la CSRF a su alcance. Lo encienden la pila de
    /// desarrollo (`compose.yaml`) y `just api-run`; una imagen de produccion que
    /// no lo pida no lo sirve aunque el gateway reenviara la ruta.
    pub swagger_ui: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 8080,
            swagger_ui: false,
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
mod tests;
