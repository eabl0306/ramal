//! Problemas de transporte: los que no nacen de un caso de uso.

use salvo::prelude::StatusCode;

use super::Problem;

/// Datos de un problema de transporte: `type`, `title`, `detail` y `code`.
type Entry = (&'static str, &'static str, &'static str, &'static str);

const NOT_FOUND: Entry = (
    "not-found",
    "No encontrado",
    "La ruta solicitada no existe en esta API.",
    "route_not_found",
);
const METHOD_NOT_ALLOWED: Entry = (
    "method-not-allowed",
    "Metodo no permitido",
    "Esa ruta no acepta este metodo HTTP.",
    "method_not_allowed",
);
const PAYLOAD_TOO_LARGE: Entry = (
    "payload-too-large",
    "Cuerpo demasiado grande",
    "El cuerpo de la peticion supera el maximo admitido.",
    "payload_too_large",
);
const UNSUPPORTED_MEDIA_TYPE: Entry = (
    "unsupported-media-type",
    "Tipo de contenido no admitido",
    "El cuerpo tiene que viajar como application/json.",
    "unsupported_media_type",
);
const TOO_MANY_REQUESTS: Entry = (
    "too-many-requests",
    "Demasiadas peticiones",
    "Has hecho demasiadas peticiones seguidas. Prueba en un momento.",
    "too_many_requests",
);
const SERVICE_UNAVAILABLE: Entry = (
    "service-unavailable",
    "Servicio no disponible",
    "El servicio no esta disponible ahora mismo. Vuelve a intentarlo en un momento.",
    "service_unavailable",
);
const BAD_REQUEST: Entry = (
    "bad-request",
    "Peticion invalida",
    "No hemos podido interpretar la peticion.",
    "bad_request",
);
const INTERNAL_ERROR: Entry = (
    "internal-error",
    "Error interno",
    "No hemos podido completar la operacion. Intentalo de nuevo.",
    "internal_error",
);

/// Traduce a problema un estado de error que produce el propio transporte.
///
/// La ruta que no existe y el metodo que no se admite los resuelve Salvo antes
/// de llegar a un handler. Sin esto saldria su pagina por defecto, que negocia el
/// `Accept` y acaba en HTML, y un cliente de esta API siempre espera JSON. El
/// estado se respeta: un 410 sigue siendo 410 aunque su `type` sea generico.
#[must_use]
pub fn problem_for_status(status: StatusCode) -> Problem {
    let (slug, title, detail, code) = entry_for(status);
    Problem::new(slug, title, status, detail, code)
}

fn entry_for(status: StatusCode) -> Entry {
    match status {
        StatusCode::NOT_FOUND => NOT_FOUND,
        StatusCode::METHOD_NOT_ALLOWED => METHOD_NOT_ALLOWED,
        StatusCode::PAYLOAD_TOO_LARGE => PAYLOAD_TOO_LARGE,
        StatusCode::UNSUPPORTED_MEDIA_TYPE => UNSUPPORTED_MEDIA_TYPE,
        StatusCode::TOO_MANY_REQUESTS => TOO_MANY_REQUESTS,
        StatusCode::SERVICE_UNAVAILABLE => SERVICE_UNAVAILABLE,
        other if other.is_client_error() => BAD_REQUEST,
        _ => INTERNAL_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use salvo::prelude::StatusCode;

    use super::problem_for_status;

    fn code(status: StatusCode) -> (u16, String) {
        let problem = problem_for_status(status);
        (problem.status, problem.code)
    }

    #[test]
    fn a_missing_route_is_a_404_with_its_own_code() {
        let problem = problem_for_status(StatusCode::NOT_FOUND);
        assert_eq!(problem.status, 404);
        assert_eq!(problem.code, "route_not_found");
        assert_eq!(problem.kind, "https://ramal.dev/problems/not-found");
        assert_eq!(problem.detail, "La ruta solicitada no existe en esta API.");
    }

    #[test]
    fn each_transport_status_has_a_stable_code() {
        assert_eq!(
            code(StatusCode::METHOD_NOT_ALLOWED),
            (405, "method_not_allowed".to_owned())
        );
        assert_eq!(
            code(StatusCode::PAYLOAD_TOO_LARGE),
            (413, "payload_too_large".to_owned())
        );
        assert_eq!(
            code(StatusCode::UNSUPPORTED_MEDIA_TYPE),
            (415, "unsupported_media_type".to_owned())
        );
        assert_eq!(
            code(StatusCode::TOO_MANY_REQUESTS),
            (429, "too_many_requests".to_owned())
        );
        assert_eq!(
            code(StatusCode::SERVICE_UNAVAILABLE),
            (503, "service_unavailable".to_owned())
        );
    }

    #[test]
    fn other_statuses_keep_their_number_with_a_generic_type() {
        assert_eq!(code(StatusCode::GONE), (410, "bad_request".to_owned()));
        assert_eq!(
            code(StatusCode::BAD_GATEWAY),
            (502, "internal_error".to_owned())
        );
    }
}
