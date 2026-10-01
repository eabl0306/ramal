#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use ramal_api_application::AppError;
use salvo::http::header::CONTENT_TYPE;
use salvo::prelude::{Response, StatusCode};

use super::{FALLBACK_BODY, PROBLEM_CONTENT_TYPE, Problem, render_problem};

#[test]
fn render_writes_status_content_type_and_body() {
    let mut res = Response::new();
    render_problem(&mut res, &AppError::not_found("workspace"));
    assert_eq!(res.status_code, Some(StatusCode::NOT_FOUND));
    assert_eq!(
        res.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some(PROBLEM_CONTENT_TYPE)
    );
    assert!(!res.body.is_none());
}

#[test]
fn an_impossible_status_falls_back_to_500() {
    let mut problem = Problem::new("x", "X", StatusCode::BAD_REQUEST, "x", "x");
    problem.status = 1;
    assert_eq!(problem.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn the_fallback_body_is_a_valid_problem() {
    let problem: Problem = serde_json::from_slice(FALLBACK_BODY).expect("valid json");
    assert_eq!(problem.status, 500);
    assert_eq!(problem.code, "internal_error");
}

#[test]
fn the_type_field_keeps_its_rfc_name_and_errors_are_omitted_when_absent() {
    let problem = Problem::new("conflict", "Conflicto", StatusCode::CONFLICT, "d", "c");
    let body = serde_json::to_value(&problem).expect("serializes");
    assert_eq!(body["type"], "https://ramal.bahoque.com/problems/conflict");
    assert!(body.get("errors").is_none());
    assert!(body.get("kind").is_none());
}
