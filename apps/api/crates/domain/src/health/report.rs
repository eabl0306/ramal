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
mod tests;
