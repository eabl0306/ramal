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
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail loudly on purpose; a panic here is the assertion"
    )]

    use ramal_api_application::Clock;
    use ramal_api_application::testing::FixedClock;
    use ramal_api_domain::{ComponentHealth, ComponentName, HealthReport, Latency, ServiceVersion};

    use super::HealthResponse;

    fn name(raw: &str) -> ComponentName {
        ComponentName::parse(raw).expect("fixture name should be valid")
    }

    fn json(report: &HealthReport) -> serde_json::Value {
        serde_json::to_value(HealthResponse::from(report)).expect("dto should serialize")
    }

    #[test]
    fn maps_a_healthy_report_to_the_public_contract() {
        let report = HealthReport::new(
            ServiceVersion::new(0, 1, 0),
            FixedClock::at_fixture().now(),
            vec![ComponentHealth::healthy(
                name("database"),
                Latency::from_millis(4),
            )],
        );
        let body = json(&report);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["version"], "0.1.0");
        assert_eq!(body["checkedAt"], "2026-09-30T10:00:00Z");
        assert_eq!(body["checks"][0]["name"], "database");
        assert_eq!(body["checks"][0]["status"], "ok");
        assert_eq!(body["checks"][0]["latencyMs"], 4);
    }

    #[test]
    fn a_degraded_report_never_publishes_the_failure_detail() {
        let report = HealthReport::new(
            ServiceVersion::new(1, 0, 0),
            FixedClock::at_fixture().now(),
            vec![ComponentHealth::failed(
                name("database"),
                Latency::from_millis(120),
                "password authentication failed for user ramal",
            )],
        );
        let body = json(&report);
        assert_eq!(body["status"], "degraded");
        assert_eq!(body["checks"][0]["status"], "error");
        assert!(body["checks"][0].get("detail").is_none());
        assert!(!body.to_string().contains("password"));
    }

    #[test]
    fn a_report_without_components_serializes_an_empty_array() {
        let report = HealthReport::without_components(
            ServiceVersion::new(2, 3, 4),
            FixedClock::at_fixture().now(),
        );
        let body = json(&report);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["checks"].as_array().map(Vec::len), Some(0));
    }
}
