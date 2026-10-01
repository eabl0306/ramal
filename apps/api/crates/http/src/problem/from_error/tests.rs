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
