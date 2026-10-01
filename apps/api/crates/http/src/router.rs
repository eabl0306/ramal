//! Composicion del router HTTP.
//!
//! El arbol es este:
//!
//! ```text
//! /health/live      publico
//! /health/ready     publico
//! /openapi.json     publico
//! /swagger-ui/      solo si se pide (RAMAL__SERVER__SWAGGER_UI): desarrollo
//! /v1/...           los contextos de dominio (ninguno en el primer corte)
//! ```
//!
//! Cada contexto tiene su sub-router en `router/<contexto>.rs` con su struct de
//! dependencias, que el composition root rellena en
//! `crates/api/src/bootstrap/<contexto>.rs`.

mod health;

use salvo::oapi::swagger_ui::SwaggerUi;
use salvo::prelude::{Router, Service};

pub use health::HealthDependencies;
use ramal_api_domain::ServiceVersion;

use crate::catcher::ProblemCatcher;
use crate::middleware::RequestId;
use crate::openapi;

/// Todo lo que el adaptador HTTP necesita para responder.
///
/// El router no construye casos de uso ni adaptadores: los recibe ya montados
/// desde el composition root.
#[derive(Debug, Clone)]
pub struct RouterDependencies {
    /// Version del servicio, que viaja en el documento OpenAPI.
    pub version: ServiceVersion,
    /// Contexto `health`.
    pub health: HealthDependencies,
    /// Monta `/swagger-ui/`. Sin el, la ruta es un 404 como cualquier otra que
    /// no existe: produccion no depende de que el gateway se acuerde de no
    /// reenviarla.
    pub swagger_ui: bool,
}

/// Rutas documentadas de la API: las que entran en `openapi.json`.
#[must_use]
pub fn api_router(dependencies: &RouterDependencies) -> Router {
    Router::new()
        .push(health::router(&dependencies.health))
        .push(v1_router(dependencies))
}

/// Monta el servicio completo: rutas, documento, middleware y catcher.
///
/// El request id va en el servicio y no en el router: los hoops del servicio
/// corren tambien cuando ninguna ruta casa, asi que un 404 o un 405 llevan su
/// `x-request-id` como cualquier otra respuesta.
#[must_use]
pub fn build_service(dependencies: &RouterDependencies) -> Service {
    let api = api_router(dependencies);
    let document = openapi::document(dependencies.version, &api);
    let mut root = Router::new()
        .push(api)
        .push(Router::with_path("openapi.json").get(document));
    if dependencies.swagger_ui {
        root = root.push(Router::with_path("swagger-ui/{**}").get(SwaggerUi::new("/openapi.json")));
    }
    Service::new(root)
        .hoop(RequestId::new())
        .catcher(ProblemCatcher::catcher())
}

/// Monta `/v1`. Cada contexto planificado se cuelga aqui con su sub-router.
fn v1_router(_dependencies: &RouterDependencies) -> Router {
    Router::with_path("v1")
}
