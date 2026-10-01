//! Traduccion de `AppError` a problema.

use ramal_api_application::{AppError, ValidationErrors};
use salvo::prelude::StatusCode;

use super::{FieldProblem, Problem};

/// Texto fijo de todos los 401, a proposito: distinguir un token que no existe
/// de uno caducado o ya usado diria si se acerto con uno que existe.
const UNAUTHORIZED_DETAIL: &str = "La sesion no es valida o ha caducado. Vuelve a iniciar sesion.";

/// Traduce un error de aplicacion a problema.
///
/// Los `reason` y `detail` internos se quedan en el log: al cliente solo le
/// llega texto pensado para el.
#[must_use]
pub fn problem_for(error: &AppError) -> Problem {
    match error {
        AppError::Validation(errors) => validation(errors),
        AppError::Unauthorized { reason } => unauthorized(reason),
        AppError::Forbidden { code, message } => Problem::new(
            "forbidden",
            "Sin permiso",
            StatusCode::FORBIDDEN,
            message,
            code,
        ),
        AppError::NotFound { resource } => not_found(resource),
        AppError::Conflict { code, message } => {
            Problem::new("conflict", "Conflicto", StatusCode::CONFLICT, message, code)
        }
        AppError::Dependency { detail } => internal(detail),
        AppError::Unavailable { detail } => unavailable(detail),
    }
}

fn validation(errors: &ValidationErrors) -> Problem {
    let mut problem = Problem::new(
        "validation",
        "Datos invalidos",
        StatusCode::UNPROCESSABLE_ENTITY,
        "Revisa los campos marcados.",
        "validation_failed",
    );
    problem.errors = errors
        .entries()
        .iter()
        .map(|entry| FieldProblem {
            field: entry.field.clone(),
            code: entry.code.clone(),
            message: entry.message.clone(),
        })
        .collect();
    problem
}

fn unauthorized(reason: &str) -> Problem {
    tracing::info!(reason = %reason, "a request was rejected as unauthorized");
    Problem::new(
        "unauthorized",
        "No autorizado",
        StatusCode::UNAUTHORIZED,
        UNAUTHORIZED_DETAIL,
        "unauthorized",
    )
}

/// El recurso buscado no se nombra en la respuesta: tambien responde asi lo que
/// es de otra cuenta, y el nombre no le sirve de nada a quien no puede verlo.
fn not_found(resource: &str) -> Problem {
    tracing::debug!(resource = %resource, "a use case reported a missing resource");
    Problem::new(
        "not-found",
        "No encontrado",
        StatusCode::NOT_FOUND,
        "El recurso solicitado no existe.",
        "not_found",
    )
}

fn internal(detail: &str) -> Problem {
    tracing::error!(detail = %detail, "a dependency failed while handling a request");
    Problem::new(
        "internal-error",
        "Error interno",
        StatusCode::INTERNAL_SERVER_ERROR,
        "No hemos podido completar la operacion. Intentalo de nuevo.",
        "internal_error",
    )
}

fn unavailable(detail: &str) -> Problem {
    tracing::warn!(detail = %detail, "a request reached a service that is not available");
    Problem::new(
        "service-unavailable",
        "Servicio no disponible",
        StatusCode::SERVICE_UNAVAILABLE,
        "El servicio no esta disponible ahora mismo. Vuelve a intentarlo en un momento.",
        "service_unavailable",
    )
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail loudly on purpose; a panic here is the assertion"
    )]

    use ramal_api_application::{AppError, FieldError, ValidationErrors};

    use super::{UNAUTHORIZED_DETAIL, problem_for};

    fn kind(error: &AppError) -> (u16, String, String) {
        let problem = problem_for(error);
        (problem.status, problem.kind, problem.code)
    }

    #[test]
    fn validation_lists_every_field_with_its_code() {
        let mut errors = ValidationErrors::new();
        errors.push(FieldError::new("name", "too_long", "Demasiado largo."));
        errors.push(FieldError::new("email", "required", "Falta el correo."));
        let problem = problem_for(&AppError::Validation(errors));
        assert_eq!(problem.status, 422);
        assert_eq!(problem.code, "validation_failed");
        let fields = problem.errors;
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].field, "email");
        assert_eq!(fields[1].code, "required");
        assert_eq!(fields[1].message, "Falta el correo.");
    }

    #[test]
    fn every_401_carries_the_same_text_whatever_the_reason() {
        let first = problem_for(&AppError::unauthorized("token expired"));
        let second = problem_for(&AppError::unauthorized("token reused"));
        assert_eq!(first, second);
        assert_eq!(first.status, 401);
        assert_eq!(first.detail, UNAUTHORIZED_DETAIL);
        assert!(!first.detail.contains("expired"));
    }

    #[test]
    fn forbidden_and_conflict_keep_their_code_and_message() {
        let forbidden = problem_for(&AppError::forbidden("insufficient_role", "Sin rol."));
        assert_eq!(
            (
                forbidden.status,
                forbidden.code.as_str(),
                forbidden.detail.as_str()
            ),
            (403, "insufficient_role", "Sin rol.")
        );
        let conflict = problem_for(&AppError::conflict("id_conflict", "Ya existe."));
        assert_eq!(
            (
                conflict.status,
                conflict.code.as_str(),
                conflict.detail.as_str()
            ),
            (409, "id_conflict", "Ya existe.")
        );
    }

    #[test]
    fn internal_details_never_reach_the_body() {
        for error in [
            AppError::not_found("secret-table"),
            AppError::dependency("connection reset by peer"),
            AppError::unavailable("pool exhausted"),
        ] {
            let body = serde_json::to_string(&problem_for(&error)).expect("serializes");
            assert!(!body.contains("secret-table"));
            assert!(!body.contains("connection reset"));
            assert!(!body.contains("pool exhausted"));
        }
    }

    #[test]
    fn each_error_maps_to_its_status_type_and_code() {
        let prefix = "https://ramal.dev/problems/";
        assert_eq!(
            kind(&AppError::not_found("x")),
            (404, format!("{prefix}not-found"), "not_found".to_owned())
        );
        assert_eq!(
            kind(&AppError::dependency("x")),
            (
                500,
                format!("{prefix}internal-error"),
                "internal_error".to_owned()
            )
        );
        assert_eq!(
            kind(&AppError::unavailable("x")),
            (
                503,
                format!("{prefix}service-unavailable"),
                "service_unavailable".to_owned()
            )
        );
    }
}
