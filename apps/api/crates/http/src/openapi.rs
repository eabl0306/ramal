//! Documento OpenAPI generado desde el propio router (salvo-oapi, ADR 0003).
//!
//! Cada handler con `#[endpoint]` registra su operacion, y el documento sale de
//! recorrer el arbol de rutas: una ruta sin documentar no puede existir. El
//! documento versionado en `apps/api/openapi.json` es la fuente de
//! `@ramal/api-client`; un test de este crate falla si se queda atras, y
//! `just api-openapi` lo reescribe.

use ramal_api_domain::ServiceVersion;
use salvo::oapi::{Components, Info, OpenApi, ToSchema};
use salvo::prelude::Router;

use crate::problem::Problem;
use crate::router::{RouterDependencies, api_router};

/// Titulo del documento.
pub const API_TITLE: &str = "Ramal API";

/// Descripcion del documento: las convenciones que valen para toda la API.
const API_DESCRIPTION: &str = "API de Ramal. Todo lo de dominio va bajo /v1. Los errores \
    salen en application/problem+json (RFC 9457) con title y detail en castellano y un \
    code estable, tambien en los 404 y 405 del transporte. Cada respuesta lleva \
    x-request-id.";

/// Construye el documento a partir de las rutas documentadas.
///
/// El esquema `Problem` se anade a mano: ningun endpoint del primer corte lo
/// devuelve en su firma, pero cualquier ruta puede responderlo (el catcher lo usa
/// en los 404 y 405) y el cliente generado necesita el tipo.
#[must_use]
pub fn document(version: ServiceVersion, router: &Router) -> OpenApi {
    let mut components = Components::new();
    let _ = Problem::to_schema(&mut components);
    let info = Info::new(API_TITLE, version.to_string()).description(API_DESCRIPTION);
    OpenApi::with_info(info)
        .components(components)
        .merge_router(router)
}

/// Documento serializado tal y como se versiona: JSON con sangria y un salto de
/// linea final, para que el diff de un cambio de contrato se lea en el PR.
///
/// # Errors
///
/// Devuelve el error de `serde_json` si el documento no se puede serializar, lo
/// que indicaria un DTO mal descrito.
pub fn document_json(dependencies: &RouterDependencies) -> Result<String, serde_json::Error> {
    let router = api_router(dependencies);
    let mut json = document(dependencies.version, &router).to_pretty_json()?;
    json.push('\n');
    Ok(json)
}
