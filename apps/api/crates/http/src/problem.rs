//! Errores HTTP en `application/problem+json` (RFC 9457).
//!
//! Es el unico sitio que convierte un error en codigo de estado y cuerpo. Un
//! handler nunca escribe un error a mano:
//!
//! ```ignore
//! match self.use_case.execute(command).await {
//!     Ok(value) => res.render(Json(Dto::from(&value))),
//!     Err(error) => render_problem(res, &error),
//! }
//! ```
//!
//! Todo problema lleva `title` y `detail` en castellano y un `code` estable, que
//! es lo que traducen los clientes con `@ramal/i18n` cuando la interfaz esta en
//! ingles.

mod from_error;
mod from_status;

use ramal_api_application::AppError;
use salvo::http::header::{CONTENT_TYPE, HeaderValue};
use salvo::oapi::ToSchema;
use salvo::prelude::{Response, StatusCode};
use serde::{Deserialize, Serialize};

pub use from_error::problem_for;
pub use from_status::problem_for_status;

/// Tipo de contenido que exige la RFC 9457.
pub const PROBLEM_CONTENT_TYPE: &str = "application/problem+json";

/// Prefijo de los `type` del contrato. Si el producto cambia de dominio, cambia
/// esto y nada mas.
pub const TYPE_PREFIX: &str = "https://ramal.bahoque.com/problems/";

/// Cuerpo de respaldo si un problema no se pudiera serializar.
const FALLBACK_BODY: &[u8] = br#"{"type":"https://ramal.bahoque.com/problems/internal-error","title":"Error interno","status":500,"detail":"No hemos podido completar la operacion. Intentalo de nuevo.","code":"internal_error"}"#;

/// Error de un campo concreto del cuerpo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "FieldProblem"))]
pub struct FieldProblem {
    /// Nombre del campo, en camelCase.
    #[salvo(schema(example = "name"))]
    pub field: String,
    /// Codigo estable del motivo.
    #[salvo(schema(example = "too_long"))]
    pub code: String,
    /// Motivo en castellano.
    #[salvo(schema(example = "El nombre no puede pasar de 60 caracteres."))]
    pub message: String,
}

/// Cuerpo de error del contrato publico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[salvo(schema(name = "Problem"))]
pub struct Problem {
    /// URI que identifica el tipo de problema.
    #[serde(rename = "type")]
    #[salvo(schema(
        rename = "type",
        example = "https://ramal.bahoque.com/problems/not-found"
    ))]
    pub kind: String,
    /// Resumen en castellano, siempre el mismo para un `type`.
    #[salvo(schema(example = "No encontrado"))]
    pub title: String,
    /// Codigo de estado HTTP, repetido en el cuerpo como pide la RFC.
    #[salvo(schema(example = 404))]
    pub status: u16,
    /// Detalle de esta ocurrencia, en castellano.
    #[salvo(schema(example = "La ruta solicitada no existe en esta API."))]
    pub detail: String,
    /// Codigo estable que distingue el motivo.
    #[salvo(schema(example = "route_not_found"))]
    pub code: String,
    /// Errores por campo. Solo aparece en los errores de validacion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FieldProblem>,
}

impl Problem {
    /// Problema sin errores de campo.
    #[must_use]
    pub fn new(slug: &str, title: &str, status: StatusCode, detail: &str, code: &str) -> Self {
        Self {
            kind: format!("{TYPE_PREFIX}{slug}"),
            title: title.to_owned(),
            status: status.as_u16(),
            detail: detail.to_owned(),
            code: code.to_owned(),
            errors: Vec::new(),
        }
    }

    /// Codigo de estado del problema; 500 si el numero guardado no es valido.
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Escribe un error de aplicacion en la respuesta.
pub fn render_problem(res: &mut Response, error: &AppError) {
    write_problem(res, &problem_for(error));
}

/// Serializa un problema ya resuelto, con el tipo de contenido de la RFC 9457.
pub fn write_problem(res: &mut Response, problem: &Problem) {
    let body = serde_json::to_vec(problem).unwrap_or_else(|_| FALLBACK_BODY.to_vec());
    res.status_code(problem.status_code());
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(PROBLEM_CONTENT_TYPE));
    // Si el cuerpo no se puede escribir, la conexion ya esta perdida: no hay
    // nada mejor que hacer que dejar el estado puesto.
    let _ = res.write_body(body);
}

#[cfg(test)]
mod tests;
