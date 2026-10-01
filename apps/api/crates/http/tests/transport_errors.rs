//! Errores que no llegan a un handler, y el request id de cada respuesta.
//!
//! La ruta que no existe y el metodo que no se admite los resuelve Salvo. Sin el
//! catcher salian en HTML, que es justo la senal que busca la prueba de humo del
//! gateway para saber que una ruta de la API acabo en la web.

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

use ramal_api_http::PROBLEM_CONTENT_TYPE;
use salvo::prelude::StatusCode;
use salvo::test::{ResponseExt, TestClient};

#[tokio::test]
async fn an_unknown_route_under_v1_answers_problem_json() {
    let mut response = TestClient::get("http://localhost/v1/ruta-que-no-existe")
        .send(&support::healthy_service())
        .await;

    assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
    assert_eq!(support::content_type(&response), PROBLEM_CONTENT_TYPE);
    let body: serde_json::Value = response.take_json().await.expect("json body");
    assert_eq!(body["type"], "https://ramal.dev/problems/not-found");
    assert_eq!(body["title"], "No encontrado");
    assert_eq!(body["status"], 404);
    assert_eq!(body["code"], "route_not_found");
    assert_eq!(body["detail"], "La ruta solicitada no existe en esta API.");
}

#[tokio::test]
async fn an_unknown_route_stays_json_when_the_client_asks_for_html() {
    let mut response = TestClient::get("http://localhost/no-existe")
        .add_header("accept", "text/html", true)
        .send(&support::healthy_service())
        .await;

    assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
    assert_eq!(support::content_type(&response), PROBLEM_CONTENT_TYPE);
    let body: serde_json::Value = response.take_json().await.expect("json body");
    assert_eq!(body["status"], 404);
}

#[tokio::test]
async fn a_method_the_route_does_not_accept_answers_405_problem_json() {
    let mut response = TestClient::delete("http://localhost/health/live")
        .send(&support::healthy_service())
        .await;

    assert_eq!(response.status_code, Some(StatusCode::METHOD_NOT_ALLOWED));
    assert_eq!(support::content_type(&response), PROBLEM_CONTENT_TYPE);
    let body: serde_json::Value = response.take_json().await.expect("json body");
    assert_eq!(
        body["type"],
        "https://ramal.dev/problems/method-not-allowed"
    );
    assert_eq!(body["status"], 405);
    assert_eq!(body["code"], "method_not_allowed");
}

#[tokio::test]
async fn every_response_carries_a_request_id_even_a_404() {
    let service = support::healthy_service();
    for path in ["/health/live", "/health/ready", "/v1/nada", "/openapi.json"] {
        let response = TestClient::get(format!("http://localhost{path}"))
            .send(&service)
            .await;
        let id = support::header(&response, "x-request-id");
        assert!(
            id.is_some_and(|value| !value.is_empty()),
            "{path} has no x-request-id"
        );
    }
}

#[tokio::test]
async fn an_incoming_request_id_is_echoed_back() {
    let response = TestClient::get("http://localhost/health/live")
        .add_header("x-request-id", "abc-123", true)
        .send(&support::healthy_service())
        .await;

    assert_eq!(
        support::header(&response, "x-request-id").as_deref(),
        Some("abc-123")
    );
}

#[tokio::test]
async fn a_strange_incoming_request_id_is_replaced() {
    let response = TestClient::get("http://localhost/health/live")
        .add_header("x-request-id", "x".repeat(200), true)
        .send(&support::healthy_service())
        .await;

    let id = support::header(&response, "x-request-id").expect("a fresh id");
    assert_eq!(id.len(), 36, "a generated id is a uuid: {id}");
}

#[tokio::test]
async fn the_api_sends_no_cors_headers() {
    let response = TestClient::get("http://localhost/health/live")
        .add_header("origin", "https://elsewhere.example", true)
        .send(&support::healthy_service())
        .await;

    assert!(support::header(&response, "access-control-allow-origin").is_none());
}
