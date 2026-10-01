//! Montaje del servicio HTTP con dobles de test, sin base de datos ni red.

#![allow(
    dead_code,
    reason = "each integration test binary compiles this module and uses only the part it needs"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use std::sync::Arc;

use ramal_api_application::testing::{FakeHealthProbe, FixedClock};
use ramal_api_application::{CheckLiveness, CheckReadiness, HealthProbe};
use ramal_api_domain::ServiceVersion;
use ramal_api_http::{HealthDependencies, RouterDependencies, build_service};
use salvo::Response;
use salvo::http::header::CONTENT_TYPE;

/// Version que anuncian las respuestas: la del workspace, la misma que pone el
/// binario. Asi el documento OpenAPI de los tests es el que se versiona.
pub fn version() -> ServiceVersion {
    ServiceVersion::parse(env!("CARGO_PKG_VERSION")).expect("the package version is semver")
}

/// Dependencias del router con las sondas dadas y un reloj fijo.
pub fn dependencies(probes: Vec<Arc<dyn HealthProbe>>) -> RouterDependencies {
    let clock = Arc::new(FixedClock::at_fixture());
    RouterDependencies {
        version: version(),
        health: HealthDependencies {
            liveness: Arc::new(CheckLiveness::new(version(), clock.clone())),
            readiness: Arc::new(CheckReadiness::new(version(), clock, probes)),
        },
    }
}

/// Servicio con una sonda de base de datos sana.
pub fn healthy_service() -> salvo::Service {
    service(vec![Arc::new(FakeHealthProbe::healthy("database"))])
}

/// Servicio con las sondas dadas.
pub fn service(probes: Vec<Arc<dyn HealthProbe>>) -> salvo::Service {
    build_service(&dependencies(probes))
}

/// Tipo de contenido de una respuesta, o cadena vacia.
pub fn content_type(response: &Response) -> String {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

/// Valor de una cabecera, o `None`.
pub fn header(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
