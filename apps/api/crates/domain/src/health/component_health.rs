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
mod tests;
