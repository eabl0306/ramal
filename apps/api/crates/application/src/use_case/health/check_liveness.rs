//! Caso de uso: comprobar que el proceso esta vivo.

use std::sync::Arc;

use ramal_api_domain::{HealthReport, ServiceVersion};

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
mod tests {
    use std::sync::Arc;

    use ramal_api_domain::{HealthStatus, ServiceVersion};

    use super::CheckLiveness;
    use crate::testing::FixedClock;

    #[test]
    fn liveness_is_always_healthy_and_has_no_components() {
        let clock = Arc::new(FixedClock::at_fixture());
        let report = CheckLiveness::new(ServiceVersion::new(1, 2, 3), clock).execute();
        assert_eq!(report.status(), HealthStatus::Ok);
        assert!(report.components().is_empty());
        assert_eq!(report.version(), ServiceVersion::new(1, 2, 3));
        assert_eq!(report.checked_at(), FixedClock::fixture_instant());
    }

    #[test]
    fn debug_shows_the_version_and_hides_the_clock() {
        let use_case = CheckLiveness::new(
            ServiceVersion::new(0, 1, 0),
            Arc::new(FixedClock::at_fixture()),
        );
        let rendered = format!("{use_case:?}");
        assert!(rendered.contains("CheckLiveness"));
        assert!(rendered.contains("version"));
    }
}
