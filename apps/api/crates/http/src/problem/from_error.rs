//! Traduccion de `AppError` a problema.

use git_rudder_api_application::{AppError, ValidationErrors};
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
mod tests;
