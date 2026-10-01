//! Sub-router de `/health`.

use std::sync::Arc;

use ramal_api_application::{CheckLiveness, CheckReadiness};
use salvo::prelude::Router;

use crate::handler::{LivenessHandler, ReadinessHandler};

/// Casos de uso del contexto `health`.
#[derive(Debug, Clone)]
pub struct HealthDependencies {
    /// Liveness: no toca dependencias.
    pub liveness: Arc<CheckLiveness>,
    /// Readiness: consulta las sondas.
    pub readiness: Arc<CheckReadiness>,
}

/// Monta `/health/live` y `/health/ready`.
#[must_use]
pub fn router(dependencies: &HealthDependencies) -> Router {
    Router::with_path("health")
        .push(Router::with_path("live").get(LivenessHandler::new(dependencies.liveness.clone())))
        .push(Router::with_path("ready").get(ReadinessHandler::new(dependencies.readiness.clone())))
}
