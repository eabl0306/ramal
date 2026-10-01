//! DTO de respuesta de `/health/live` y `/health/ready`.

use ramal_api_domain::{ComponentHealth, ComponentStatus, HealthReport, HealthStatus};
use salvo::oapi::ToSchema;
use serde::Serialize;

/// Formato de los instantes del contrato: RFC 3339 en UTC, con `Z`.
const INSTANT_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

/// Estado global del servicio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[salvo(schema(name = "HealthState"))]
pub enum HealthState {
    /// Todos los componentes responden.
    Ok,
    /// Algun componente no responde.
    Degraded,
}

/// Estado de un componente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[salvo(schema(name = "CheckState"))]
pub enum CheckState {
    /// El componente respondio bien.
    Ok,
    /// El componente no respondio o respondio mal.
    Error,
}

/// Lectura de un componente. El motivo de un fallo no viaja: queda en el log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[salvo(schema(name = "HealthCheck"))]
pub struct HealthCheck {
    /// Nombre del componente (`database`).
    #[salvo(schema(example = "database"))]
    pub name: String,
    /// Estado del componente.
    pub status: CheckState,
    /// Cuanto tardo en responder, en milisegundos.
    #[salvo(schema(example = 3))]
    pub latency_ms: u64,
}

/// Respuesta de `/health/live` y `/health/ready`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[salvo(schema(name = "HealthResponse"))]
pub struct HealthResponse {
    /// Estado global.
    pub status: HealthState,
    /// Una lectura por componente comprobado; vacio en liveness.
    pub checks: Vec<HealthCheck>,
    /// Version del servicio que responde.
    #[salvo(schema(example = "0.1.0"))]
    pub version: String,
    /// Instante de la comprobacion.
    #[salvo(schema(format = DateTime, example = "2026-09-30T10:00:00Z"))]
    pub checked_at: String,
}

impl From<HealthStatus> for HealthState {
    fn from(status: HealthStatus) -> Self {
        match status {
            HealthStatus::Ok => Self::Ok,
            HealthStatus::Degraded => Self::Degraded,
        }
    }
}

impl From<ComponentStatus> for CheckState {
    fn from(status: ComponentStatus) -> Self {
        match status {
            ComponentStatus::Ok => Self::Ok,
            ComponentStatus::Error => Self::Error,
        }
    }
}

impl From<&ComponentHealth> for HealthCheck {
    fn from(component: &ComponentHealth) -> Self {
        Self {
            name: component.name().as_str().to_owned(),
            status: component.status().into(),
            latency_ms: component.latency().as_millis(),
        }
    }
}

impl From<&HealthReport> for HealthResponse {
    fn from(report: &HealthReport) -> Self {
        Self {
            status: report.status().into(),
            checks: report.components().iter().map(Into::into).collect(),
            version: report.version().to_string(),
            checked_at: report.checked_at().format(INSTANT_FORMAT).to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
