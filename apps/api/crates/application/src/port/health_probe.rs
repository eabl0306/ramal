//! Puerto para comprobar si una dependencia del servicio responde.

use async_trait::async_trait;
use git_rudder_api_domain::ComponentName;
use thiserror::Error;

/// Fallo de una sonda: la dependencia no respondio, o respondio mal.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ProbeError {
    message: String,
}

impl ProbeError {
    /// Crea el fallo con el motivo que vio el adaptador.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Motivo del fallo, en ingles y para el log.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Comprueba que una dependencia responde.
///
/// La implementa cada adaptador que el servicio necesita para recibir trafico:
/// hoy PostgreSQL (`database`) y, con las llaves de IA de organizacion, la llave
/// maestra (`secrets`). El adaptador solo dice si la dependencia contesta; la
/// latencia la mide el caso de uso y la regla de degradacion es del dominio.
#[async_trait]
pub trait HealthProbe: Send + Sync {
    /// Nombre del componente que observa esta sonda.
    fn name(&self) -> &ComponentName;

    /// Comprueba la dependencia.
    ///
    /// # Errors
    ///
    /// Devuelve [`ProbeError`] si la dependencia no responde o responde mal.
    async fn check(&self) -> Result<(), ProbeError>;
}

#[cfg(test)]
mod tests;
