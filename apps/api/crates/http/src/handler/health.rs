//! Handlers de `/health/live` y `/health/ready`.

use std::sync::Arc;

use git_rudder_api_application::{CheckLiveness, CheckReadiness};
use git_rudder_api_domain::{ComponentHealth, HealthReport};
use salvo::oapi::endpoint;
use salvo::prelude::{Json, Response, StatusCode};

use crate::dto::HealthResponse;

/// Responde a `/health/live`.
#[derive(Debug, Clone)]
pub struct LivenessHandler {
    use_case: Arc<CheckLiveness>,
}

impl LivenessHandler {
    /// Crea el handler sobre su caso de uso.
    #[must_use]
    pub const fn new(use_case: Arc<CheckLiveness>) -> Self {
        Self { use_case }
    }
}

/// El proceso esta vivo
///
/// Responde 200 mientras el proceso atienda, sin consultar ninguna dependencia.
/// `checks` siempre va vacio.
#[endpoint(
    operation_id = "checkLiveness",
    tags("health"),
    responses((status_code = 200, description = "El proceso atiende.", body = HealthResponse))
)]
impl LivenessHandler {
    fn handle(&self, res: &mut Response) {
        write_report(res, &self.use_case.execute());
    }
}

/// Responde a `/health/ready`.
#[derive(Debug, Clone)]
pub struct ReadinessHandler {
    use_case: Arc<CheckReadiness>,
}

impl ReadinessHandler {
    /// Crea el handler sobre su caso de uso.
    #[must_use]
    pub const fn new(use_case: Arc<CheckReadiness>) -> Self {
        Self { use_case }
    }
}

/// El servicio puede recibir trafico
///
/// 200 con `status: ok` cuando todos los componentes responden; 503 con
/// `status: degraded` en cuanto uno falla. El motivo del fallo queda en el log
/// del servidor y no viaja en la respuesta.
#[endpoint(
    operation_id = "checkReadiness",
    tags("health"),
    responses(
        (status_code = 200, description = "Todos los componentes responden.", body = HealthResponse),
        (status_code = 503, description = "Algun componente no responde.", body = HealthResponse)
    )
)]
impl ReadinessHandler {
    async fn handle(&self, res: &mut Response) {
        let report = self.use_case.execute().await;
        if !report.is_healthy() {
            log_degraded(&report.failing_components());
        }
        write_report(res, &report);
    }
}

fn write_report(res: &mut Response, report: &HealthReport) {
    let status = if report.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    res.status_code(status);
    res.render(Json(HealthResponse::from(report)));
}

/// El detalle de cada fallo solo existe aqui, en el log.
fn log_degraded(failing: &[&ComponentHealth]) {
    for component in failing {
        tracing::warn!(
            component = %component.name(),
            latency_ms = component.latency().as_millis(),
            detail = component.detail().unwrap_or_default(),
            "readiness check found a failing component"
        );
    }
}
