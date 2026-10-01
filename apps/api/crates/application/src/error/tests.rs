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
    let AppError::Validation(errors) = AppError::validation("email", "required", "Falta.") else {
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
