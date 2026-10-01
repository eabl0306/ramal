//! Caso de uso: comprobar que el servicio puede recibir trafico.

use std::sync::Arc;
use std::time::Instant;

use ramal_api_domain::{ComponentHealth, HealthReport, Latency, ServiceVersion};

use crate::port::{Clock, HealthProbe};

/// Sondas que consulta el caso de uso, en el orden en que se publican.
pub type Probes = Vec<Arc<dyn HealthProbe>>;

/// Ejecuta todas las sondas y compone el informe de salud.
///
/// No sabe que hay detras de cada sonda: mide cuanto tarda, apunta el motivo si
/// falla y deja que el dominio decida si el servicio queda degradado. Las sondas
/// van en serie: son pocas y cada una lleva su propio timeout.
#[derive(Clone)]
pub struct CheckReadiness {
    version: ServiceVersion,
    clock: Arc<dyn Clock>,
    probes: Probes,
}

impl CheckReadiness {
    /// Crea el caso de uso con las sondas que debe consultar.
    #[must_use]
    pub fn new(version: ServiceVersion, clock: Arc<dyn Clock>, probes: Probes) -> Self {
        Self {
            version,
            clock,
            probes,
        }
    }

    /// Consulta las sondas en orden y devuelve el informe.
    pub async fn execute(&self) -> HealthReport {
        let checked_at = self.clock.now();
        let mut components = Vec::with_capacity(self.probes.len());
        for probe in &self.probes {
            components.push(Self::run(probe.as_ref()).await);
        }
        HealthReport::new(self.version, checked_at, components)
    }

    async fn run(probe: &dyn HealthProbe) -> ComponentHealth {
        let started = Instant::now();
        let outcome = probe.check().await;
        let latency = Latency::from_duration(started.elapsed());
        let name = probe.name().clone();
        match outcome {
            Ok(()) => ComponentHealth::healthy(name, latency),
            Err(error) => ComponentHealth::failed(name, latency, error.message()),
        }
    }
}

impl std::fmt::Debug for CheckReadiness {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CheckReadiness")
            .field("version", &self.version)
            .field("probes", &self.probes.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
