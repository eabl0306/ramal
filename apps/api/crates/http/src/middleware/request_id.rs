//! Middleware que garantiza un identificador por peticion.

use salvo::http::header::HeaderValue;
use salvo::prelude::{Depot, FlowCtrl, Handler, Request, Response};
use uuid::Uuid;

/// Cabecera que transporta el identificador de peticion.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Clave con la que el identificador se guarda en el `Depot`.
pub const REQUEST_ID_KEY: &str = "request_id";

/// Longitud maxima de un identificador que llega de fuera.
const MAX_INCOMING_LENGTH: usize = 128;

/// Rutas de las sondas de salud.
const HEALTH_PREFIX: &str = "/health/";

/// Propaga el `x-request-id` entrante o genera uno nuevo.
///
/// Tambien deja una traza por peticion con metodo, ruta y estado, que es lo
/// minimo para correlacionar los logs del gateway, de la web, del motor y de la
/// API. Un identificador entrante raro (vacio, enorme o con caracteres que no
/// caben en una cabecera) se sustituye: no se copia a los logs nada que no se
/// haya validado.
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestId;

impl RequestId {
    /// Crea el middleware.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[salvo::async_trait]
impl Handler for RequestId {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let request_id = incoming_id(req).unwrap_or_else(|| Uuid::new_v4().to_string());
        let method = req.method().to_string();
        let path = req.uri().path().to_owned();
        depot.insert(REQUEST_ID_KEY, request_id.clone());

        ctrl.call_next(req, depot, res).await;

        if let Ok(value) = HeaderValue::from_str(&request_id) {
            res.headers_mut().insert(REQUEST_ID_HEADER, value);
        }
        let status = res.status_code.map_or(0, |code| code.as_u16());
        log_request(&request_id, &method, &path, status);
    }
}

/// Las sondas de salud llegan cada pocos segundos desde docker y desde el
/// orquestador: en `info` taparian el resto del log, asi que van en `debug`.
fn log_request(request_id: &str, method: &str, path: &str, status: u16) {
    if path.starts_with(HEALTH_PREFIX) {
        log::probe(request_id, method, path, status);
    } else {
        log::request(request_id, method, path, status);
    }
}

/// Cada macro de `tracing` expande a bastante codigo: juntas en una funcion con
/// un `if` pasan del limite de complejidad cognitiva.
mod log {
    pub fn probe(request_id: &str, method: &str, path: &str, status: u16) {
        tracing::debug!(request_id, method, path, status, "handled request");
    }

    pub fn request(request_id: &str, method: &str, path: &str, status: u16) {
        tracing::info!(request_id, method, path, status, "handled request");
    }
}

fn incoming_id(req: &Request) -> Option<String> {
    let raw = req.headers().get(REQUEST_ID_HEADER)?.to_str().ok()?.trim();
    is_acceptable(raw).then(|| raw.to_owned())
}

fn is_acceptable(raw: &str) -> bool {
    !raw.is_empty()
        && raw.len() <= MAX_INCOMING_LENGTH
        && raw.bytes().all(|byte| byte.is_ascii_graphic())
}

#[cfg(test)]
mod tests {
    use super::{MAX_INCOMING_LENGTH, is_acceptable, log_request};

    #[test]
    fn accepts_a_reasonable_identifier() {
        assert!(is_acceptable("abc-123"));
        assert!(is_acceptable("01J9Z3Q4W5E6R7T8Y9U0I1O2P3"));
    }

    #[test]
    fn rejects_empty_long_or_strange_identifiers() {
        assert!(!is_acceptable(""));
        assert!(!is_acceptable(&"a".repeat(MAX_INCOMING_LENGTH + 1)));
        assert!(!is_acceptable("two words"));
        assert!(!is_acceptable("tab\there"));
    }

    #[test]
    fn logging_a_probe_or_a_request_never_panics() {
        log_request("id", "GET", "/health/live", 200);
        log_request("id", "GET", "/v1/workspaces", 404);
    }
}
