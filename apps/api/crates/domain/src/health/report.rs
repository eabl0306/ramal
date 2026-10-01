//! Informe que decide si el servicio esta sano.

use chrono::{DateTime, Utc};

use crate::health::component_health::ComponentHealth;
use crate::value_object::{ComponentName, ServiceVersion};

/// Estado global del servicio tal y como lo expone el contrato publico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthStatus {
    /// Todos los componentes comprobados responden.
    Ok,
    /// Al menos un componente comprobado esta caido.
    Degraded,
}

impl HealthStatus {
    /// Indica si el servicio puede aceptar trafico.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Valor estable del contrato publico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Degraded => "degraded",
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Informe de salud del servicio en un instante.
///
/// La regla vive aqui: basta un componente caido para que el informe este
/// degradado. Un informe sin componentes (el de liveness) siempre esta sano,
/// porque no depende de nada externo. El instante lo pone quien construye el
/// informe con el puerto `Clock`: el dominio no lee la hora.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    version: ServiceVersion,
    checked_at: DateTime<Utc>,
    components: Vec<ComponentHealth>,
}

impl HealthReport {
    /// Informe a partir de las lecturas de sus componentes.
    #[must_use]
    pub const fn new(
        version: ServiceVersion,
        checked_at: DateTime<Utc>,
        components: Vec<ComponentHealth>,
    ) -> Self {
        Self {
            version,
            checked_at,
            components,
        }
    }

    /// Informe sin dependencias externas.
    #[must_use]
    pub const fn without_components(version: ServiceVersion, checked_at: DateTime<Utc>) -> Self {
        Self::new(version, checked_at, Vec::new())
    }

    /// Version del servicio que emitio el informe.
    #[must_use]
    pub const fn version(&self) -> ServiceVersion {
        self.version
    }

    /// Instante en el que se hizo la comprobacion.
    #[must_use]
    pub const fn checked_at(&self) -> DateTime<Utc> {
        self.checked_at
    }

    /// Lecturas de los componentes comprobados.
    #[must_use]
    pub fn components(&self) -> &[ComponentHealth] {
        &self.components
    }

    /// Estado global derivado de las lecturas.
    #[must_use]
    pub fn status(&self) -> HealthStatus {
        if self
            .components
            .iter()
            .all(|component| component.status().is_ok())
        {
            HealthStatus::Ok
        } else {
            HealthStatus::Degraded
        }
    }

    /// Atajo para saber si el servicio esta sano.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.status().is_ok()
    }

    /// Componentes que estan fallando, en el orden en que se comprobaron.
    #[must_use]
    pub fn failing_components(&self) -> Vec<&ComponentHealth> {
        self.components
            .iter()
            .filter(|component| !component.status().is_ok())
            .collect()
    }

    /// Nombres de los componentes que estan fallando.
    #[must_use]
    pub fn failing_names(&self) -> Vec<&ComponentName> {
        self.failing_components()
            .into_iter()
            .map(ComponentHealth::name)
            .collect()
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

    use chrono::{DateTime, Utc};

    use super::{HealthReport, HealthStatus};
    use crate::health::ComponentHealth;
    use crate::value_object::{ComponentName, Latency, ServiceVersion};

    fn name(raw: &str) -> ComponentName {
        ComponentName::parse(raw).expect("fixture name should be valid")
    }

    fn version() -> ServiceVersion {
        ServiceVersion::new(0, 1, 0)
    }

    fn instant() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-30T10:00:00Z")
            .expect("fixture instant should be valid")
            .with_timezone(&Utc)
    }

    #[test]
    fn a_report_without_components_is_healthy() {
        let report = HealthReport::without_components(version(), instant());
        assert_eq!(report.status(), HealthStatus::Ok);
        assert!(report.is_healthy());
        assert!(report.components().is_empty());
        assert_eq!(report.version(), version());
        assert_eq!(report.checked_at(), instant());
    }

    #[test]
    fn all_components_ok_keep_the_report_healthy() {
        let report = HealthReport::new(
            version(),
            instant(),
            vec![
                ComponentHealth::healthy(name("database"), Latency::from_millis(3)),
                ComponentHealth::healthy(name("secrets"), Latency::from_millis(1)),
            ],
        );
        assert_eq!(report.status(), HealthStatus::Ok);
        assert!(report.failing_components().is_empty());
    }

    #[test]
    fn a_single_failing_component_degrades_the_whole_report() {
        let report = HealthReport::new(
            version(),
            instant(),
            vec![
                ComponentHealth::healthy(name("secrets"), Latency::from_millis(1)),
                ComponentHealth::failed(name("database"), Latency::from_millis(50), "timeout"),
            ],
        );
        assert_eq!(report.status(), HealthStatus::Degraded);
        assert!(!report.is_healthy());
        let failing = report.failing_components();
        assert_eq!(failing.len(), 1);
        assert_eq!(failing[0].detail(), Some("timeout"));
        let names: Vec<&str> = report.failing_names().iter().map(|n| n.as_str()).collect();
        assert_eq!(names, vec!["database"]);
    }

    #[test]
    fn status_renders_the_public_contract_values() {
        assert_eq!(HealthStatus::Ok.to_string(), "ok");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert!(HealthStatus::Ok.is_ok());
        assert!(!HealthStatus::Degraded.is_ok());
    }
}
