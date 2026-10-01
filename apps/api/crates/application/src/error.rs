//! Error unico de la capa de aplicacion.
//!
//! Todos los casos de uso que pueden fallar devuelven este error, y el adaptador
//! HTTP lo traduce una sola vez a `application/problem+json`. Asi el mapeo a
//! codigos de estado vive en un sitio y no repartido por los handlers.
//!
//! Los `code` son estables: los clientes los traducen con `@ramal/i18n` cuando la
//! interfaz esta en ingles. Los `message` van en castellano y son el texto de
//! respaldo. Los `reason` y `detail` son solo para el log y nunca viajan.

use thiserror::Error;

/// Error de validacion de un campo concreto del cuerpo de la peticion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// Nombre del campo en el JSON, en camelCase.
    pub field: String,
    /// Codigo estable del motivo, en snake_case (`too_long`, `required`...).
    pub code: String,
    /// Mensaje en castellano para el cliente.
    pub message: String,
}

impl FieldError {
    /// Crea un error de campo.
    #[must_use]
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

/// Errores de validacion, uno por campo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationErrors(Vec<FieldError>);

impl ValidationErrors {
    /// Coleccion vacia.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Anade un error de campo.
    pub fn push(&mut self, error: FieldError) {
        self.0.push(error);
    }

    /// Indica si no hay ningun error.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Errores recogidos, en el orden en que se anadieron.
    #[must_use]
    pub fn entries(&self) -> &[FieldError] {
        &self.0
    }

    /// `Ok(())` si no hay errores.
    ///
    /// # Errors
    ///
    /// Devuelve [`AppError::Validation`] con los errores recogidos.
    pub fn into_result(self) -> Result<(), AppError> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(self))
        }
    }
}

/// Fallo de un caso de uso.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AppError {
    /// El cuerpo no cumple las reglas. Se traduce a 422.
    #[error("the request body is invalid")]
    Validation(ValidationErrors),
    /// Falta la identidad, o no vale. Se traduce a 401 con un texto fijo.
    ///
    /// El `reason` es para el log: distinguir un token que no existe de uno
    /// caducado le diria a quien prueba si acerto con uno que existe.
    #[error("unauthorized: {reason}")]
    Unauthorized {
        /// Motivo, solo para la traza del servidor.
        reason: String,
    },
    /// El recurso es visible pero falta permiso. Se traduce a 403.
    #[error("{message}")]
    Forbidden {
        /// Codigo estable (`insufficient_role`, `reauthentication_required`...).
        code: String,
        /// Mensaje en castellano.
        message: String,
    },
    /// El recurso no existe, o es de otra cuenta. Se traduce a 404.
    ///
    /// Tambien cuando es de otra cuenta: un 403 confirmaria que existe.
    #[error("{resource} not found")]
    NotFound {
        /// Recurso buscado, para el log.
        resource: String,
    },
    /// La operacion choca con una regla de negocio. Se traduce a 409.
    #[error("{message}")]
    Conflict {
        /// Codigo estable para que el cliente distinga el motivo.
        code: String,
        /// Mensaje en castellano.
        message: String,
    },
    /// Una dependencia fallo. Se traduce a 500 sin contar detalles.
    #[error("a dependency failed: {detail}")]
    Dependency {
        /// Detalle tecnico, solo para la traza del servidor.
        detail: String,
    },
    /// El servicio no puede atender esto ahora mismo. Se traduce a 503.
    #[error("unavailable: {detail}")]
    Unavailable {
        /// Detalle tecnico, solo para la traza del servidor.
        detail: String,
    },
}

impl AppError {
    /// Error de validacion de un solo campo.
    #[must_use]
    pub fn validation(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let mut errors = ValidationErrors::new();
        errors.push(FieldError::new(field, code, message));
        Self::Validation(errors)
    }

    /// Identidad ausente o invalida.
    #[must_use]
    pub fn unauthorized(reason: impl Into<String>) -> Self {
        Self::Unauthorized {
            reason: reason.into(),
        }
    }

    /// Falta permiso sobre algo que el usuario si puede ver.
    #[must_use]
    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Forbidden {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Recurso inexistente o ajeno.
    #[must_use]
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Conflicto con una regla de negocio.
    #[must_use]
    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Conflict {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Fallo de una dependencia.
    #[must_use]
    pub fn dependency(detail: impl Into<String>) -> Self {
        Self::Dependency {
            detail: detail.into(),
        }
    }

    /// Servicio no disponible por ahora.
    #[must_use]
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self::Unavailable {
            detail: detail.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail loudly on purpose; a panic here is the assertion"
    )]

    use super::{AppError, FieldError, ValidationErrors};

    #[test]
    fn an_empty_collection_is_not_an_error() {
        assert_eq!(ValidationErrors::new().into_result(), Ok(()));
    }

    #[test]
    fn a_collection_with_entries_becomes_a_validation_error() {
        let mut errors = ValidationErrors::new();
        errors.push(FieldError::new("name", "too_long", "Demasiado largo."));
        assert!(!errors.is_empty());
        let Err(AppError::Validation(collected)) = errors.into_result() else {
            unreachable!("a non empty collection must fail");
        };
        assert_eq!(collected.entries()[0].code, "too_long");
        assert_eq!(collected.entries()[0].field, "name");
    }

    #[test]
    fn the_single_field_shortcut_keeps_field_code_and_message() {
        let AppError::Validation(errors) = AppError::validation("email", "required", "Falta.")
        else {
            unreachable!("the shortcut must build a validation error");
        };
        assert_eq!(
            errors.entries(),
            &[FieldError::new("email", "required", "Falta.")]
        );
    }

    #[test]
    fn constructors_fill_their_variant() {
        assert_eq!(
            AppError::unauthorized("expired"),
            AppError::Unauthorized {
                reason: "expired".to_owned()
            }
        );
        assert_eq!(
            AppError::forbidden("insufficient_role", "Sin permiso."),
            AppError::Forbidden {
                code: "insufficient_role".to_owned(),
                message: "Sin permiso.".to_owned()
            }
        );
        assert_eq!(
            AppError::not_found("workspace"),
            AppError::NotFound {
                resource: "workspace".to_owned()
            }
        );
        assert_eq!(
            AppError::conflict("id_conflict", "Ya existe."),
            AppError::Conflict {
                code: "id_conflict".to_owned(),
                message: "Ya existe.".to_owned()
            }
        );
        assert_eq!(
            AppError::dependency("pool closed"),
            AppError::Dependency {
                detail: "pool closed".to_owned()
            }
        );
        assert_eq!(
            AppError::unavailable("warming up"),
            AppError::Unavailable {
                detail: "warming up".to_owned()
            }
        );
    }

    #[test]
    fn display_is_meant_for_the_server_log() {
        assert_eq!(
            AppError::not_found("workspace").to_string(),
            "workspace not found"
        );
        assert_eq!(
            AppError::dependency("boom").to_string(),
            "a dependency failed: boom"
        );
        assert_eq!(
            AppError::validation("a", "b", "c").to_string(),
            "the request body is invalid"
        );
    }
}
