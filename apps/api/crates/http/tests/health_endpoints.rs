//! `/health/live` y `/health/ready` contra dobles de test.
//!
//! `salvo::test::TestClient` atraviesa el servicio entero sin abrir ningun
//! puerto: middleware, router, handler, caso de uso, DTO y catcher.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]
#![allow(
    unused_crate_dependencies,
    reason = "the lint runs per target and an integration test never uses every dependency"
)]

mod support;

use std::sync::Arc;

use git_rudder_api_application::testing::FakeHealthProbe;
use salvo::prelude::StatusCode;
use salvo::test::{ResponseExt, TestClient};

async fn get_json(
    service: &salvo::Service,
    path: &str,
) -> (Option<StatusCode>, String, serde_json::Value) {
    let mut response = TestClient::get(format!("http://localhost{path}"))
        .send(service)
        .await;
    let content_type = support::content_type(&response);
    let body = response.take_json().await.expect("json body");
    (response.status_code, content_type, body)
}

#[tokio::test]
async fn liveness_is_ok_even_when_a_dependency_is_down() {
    let service = support::service(vec![Arc::new(FakeHealthProbe::failing("database", "down"))]);
    let (status, content_type, body) = get_json(&service, "/health/live").await;

    assert_eq!(status, Some(StatusCode::OK));
    assert!(content_type.starts_with("application/json"));
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], support::version().to_string());
    assert_eq!(body["checkedAt"], "2026-09-30T10:00:00Z");
    assert_eq!(body["checks"].as_array().map(Vec::len), Some(0));
}

#[tokio::test]
async fn readiness_is_ok_when_every_probe_answers() {
    let (status, _, body) = get_json(&support::healthy_service(), "/health/ready").await;

    assert_eq!(status, Some(StatusCode::OK));
    assert_eq!(body["status"], "ok");
    assert_eq!(body["checks"][0]["name"], "database");
    assert_eq!(body["checks"][0]["status"], "ok");
    assert!(body["checks"][0]["latencyMs"].is_u64());
    assert_eq!(body["checkedAt"], "2026-09-30T10:00:00Z");
}

#[tokio::test]
async fn readiness_answers_503_degraded_when_a_probe_fails() {
    let service = support::service(vec![Arc::new(FakeHealthProbe::failing(
        "database",
        "password authentication failed",
    ))]);
    let (status, content_type, body) = get_json(&service, "/health/ready").await;

    assert_eq!(status, Some(StatusCode::SERVICE_UNAVAILABLE));
    assert!(
        content_type.starts_with("application/json"),
        "a degraded report is still a report, not a problem: got {content_type}"
    );
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["checks"][0]["status"], "error");
    assert!(!body.to_string().contains("password"));
}

#[tokio::test]
async fn one_failing_probe_among_many_degrades_the_service() {
    let service = support::service(vec![
        Arc::new(FakeHealthProbe::healthy("secrets")),
        Arc::new(FakeHealthProbe::failing("database", "timeout")),
    ]);
    let (status, _, body) = get_json(&service, "/health/ready").await;

    assert_eq!(status, Some(StatusCode::SERVICE_UNAVAILABLE));
    assert_eq!(body["checks"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["checks"][0]["status"], "ok");
    assert_eq!(body["checks"][1]["status"], "error");
}

#[tokio::test]
async fn each_readiness_request_runs_the_probes_again() {
    let probe = Arc::new(FakeHealthProbe::healthy("database"));
    let service = support::service(vec![probe.clone()]);
    get_json(&service, "/health/ready").await;
    get_json(&service, "/health/ready").await;

    assert_eq!(probe.calls(), 2);
}
