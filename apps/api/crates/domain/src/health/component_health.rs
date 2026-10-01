//! Salud observada de un componente concreto.

use crate::value_object::{ComponentName, Latency};

/// Resultado de comprobar un componente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentStatus {
    /// El componente respondio bien.
    Ok,
    /// El componente no respondio o respondio mal.
    Error,
}

impl ComponentStatus {
    /// Indica si el componente esta operativo.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Valor estable del contrato publico.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Lectura de un componente: estado, latencia y, si fallo, el motivo.
///
/// El motivo es para el log del servidor. El adaptador HTTP no lo publica: un
/// mensaje de error de PostgreSQL le cuenta a cualquiera como esta montado el
/// servicio por dentro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentHealth {
    name: ComponentName,
    status: ComponentStatus,
    latency: Latency,
    detail: Option<String>,
}

impl ComponentHealth {
    /// Lectura correcta.
    #[must_use]
    pub const fn healthy(name: ComponentName, latency: Latency) -> Self {
        Self {
            name,
            status: ComponentStatus::Ok,
            latency,
            detail: None,
        }
    }

    /// Lectura fallida con el motivo observado.
    #[must_use]
    pub fn failed(name: ComponentName, latency: Latency, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: ComponentStatus::Error,
            latency,
            detail: Some(detail.into()),
        }
    }

    /// Nombre del componente.
    #[must_use]
    pub const fn name(&self) -> &ComponentName {
        &self.name
    }

    /// Estado observado.
    #[must_use]
    pub const fn status(&self) -> ComponentStatus {
        self.status
    }

    /// Latencia de la comprobacion.
    #[must_use]
    pub const fn latency(&self) -> Latency {
        self.latency
    }

    /// Motivo del fallo, si lo hubo.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
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

    use super::{ComponentHealth, ComponentStatus};
    use crate::value_object::{ComponentName, Latency};

    fn database() -> ComponentName {
        ComponentName::parse("database").expect("fixture name should be valid")
    }

    #[test]
    fn a_healthy_reading_has_no_detail() {
        let reading = ComponentHealth::healthy(database(), Latency::from_millis(7));
        assert_eq!(reading.status(), ComponentStatus::Ok);
        assert_eq!(reading.latency().as_millis(), 7);
        assert_eq!(reading.detail(), None);
        assert_eq!(reading.name().as_str(), "database");
    }

    #[test]
    fn a_failed_reading_keeps_the_reason() {
        let reading =
            ComponentHealth::failed(database(), Latency::from_millis(30), "connection refused");
        assert_eq!(reading.status(), ComponentStatus::Error);
        assert_eq!(reading.detail(), Some("connection refused"));
    }

    #[test]
    fn status_renders_the_public_contract_values() {
        assert_eq!(ComponentStatus::Ok.to_string(), "ok");
        assert_eq!(ComponentStatus::Error.to_string(), "error");
        assert!(ComponentStatus::Ok.is_ok());
        assert!(!ComponentStatus::Error.is_ok());
    }
}
