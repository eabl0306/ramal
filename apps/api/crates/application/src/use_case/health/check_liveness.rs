//! Caso de uso: comprobar que el proceso esta vivo.

use std::sync::Arc;

use git_rudder_api_domain::{HealthReport, ServiceVersion};

use crate::port::Clock;

/// Responde si el proceso atiende, sin tocar ninguna dependencia.
///
/// Liveness nunca consulta componentes externos: si un orquestador reiniciara el
/// contenedor porque PostgreSQL esta caido, el problema solo empeoraria.
#[derive(Clone)]
pub struct CheckLiveness {
    version: ServiceVersion,
    clock: Arc<dyn Clock>,
}

impl CheckLiveness {
    /// Crea el caso de uso.
    #[must_use]
    pub fn new(version: ServiceVersion, clock: Arc<dyn Clock>) -> Self {
        Self { version, clock }
    }

    /// Devuelve un informe siempre sano.
    #[must_use]
    pub fn execute(&self) -> HealthReport {
        HealthReport::without_components(self.version, self.clock.now())
    }
}

impl std::fmt::Debug for CheckLiveness {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CheckLiveness")
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
