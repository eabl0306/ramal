//! El documento OpenAPI: se sirve, describe las rutas y el versionado esta al dia.

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

use std::path::PathBuf;

use ramal_api_http::{API_TITLE, document_json};
use salvo::prelude::StatusCode;
use salvo::test::{ResponseExt, TestClient};

/// Documento versionado, fuente de `@ramal/api-client`.
fn committed_document_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("openapi.json")
}

fn generated_document() -> String {
    document_json(&support::dependencies(Vec::new())).expect("the document serializes")
}

#[test]
fn the_committed_document_matches_the_router() {
    let committed = std::fs::read_to_string(committed_document_path())
        .expect("apps/api/openapi.json should exist: run `just api-openapi`");
    assert!(
        committed == generated_document(),
        "apps/api/openapi.json is stale: the router describes a different contract. \
         Run `just api-openapi`, then `just api-client`, and commit both."
    );
}

#[test]
fn the_document_describes_the_health_routes_and_the_problem_schema() {
    let document: serde_json::Value =
        serde_json::from_str(&generated_document()).expect("valid json");

    assert_eq!(document["info"]["title"], API_TITLE);
    assert_eq!(document["info"]["version"], support::version().to_string());
    let live = &document["paths"]["/health/live"]["get"];
    assert_eq!(live["operationId"], "checkLiveness");
    let ready = &document["paths"]["/health/ready"]["get"];
    assert_eq!(ready["operationId"], "checkReadiness");
    assert!(ready["responses"]["503"].is_object());
    let schemas = &document["components"]["schemas"];
    for name in ["HealthResponse", "HealthCheck", "Problem", "FieldProblem"] {
        assert!(schemas[name].is_object(), "schema {name} is missing");
    }
}

#[test]
fn the_document_does_not_describe_its_own_viewers() {
    let document: serde_json::Value =
        serde_json::from_str(&generated_document()).expect("valid json");
    let paths = document["paths"].as_object().expect("paths is an object");

    assert!(!paths.contains_key("/openapi.json"));
    assert!(paths.keys().all(|path| !path.starts_with("/swagger-ui")));
}

#[tokio::test]
async fn the_document_is_served_at_openapi_json() {
    let mut response = TestClient::get("http://localhost/openapi.json")
        .send(&support::healthy_service())
        .await;

    assert_eq!(response.status_code, Some(StatusCode::OK));
    let served: serde_json::Value = response.take_json().await.expect("json body");
    let generated: serde_json::Value =
        serde_json::from_str(&generated_document()).expect("valid json");
    assert_eq!(served, generated);
}

#[tokio::test]
async fn swagger_ui_is_served_without_a_cdn_when_enabled() {
    let mut response = TestClient::get("http://localhost/swagger-ui/")
        .send(&support::service_with_swagger_ui())
        .await;

    assert_eq!(response.status_code, Some(StatusCode::OK));
    assert!(support::content_type(&response).starts_with("text/html"));
    let page = response.take_string().await.expect("html body");
    assert!(page.contains("/openapi.json"));
    assert!(!page.contains("cdn.jsdelivr.net"));
}

#[tokio::test]
async fn swagger_ui_is_a_plain_404_when_disabled() {
    let mut response = TestClient::get("http://localhost/swagger-ui/")
        .send(&support::healthy_service())
        .await;

    assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
    assert!(support::content_type(&response).starts_with("application/problem+json"));
    let body = response.take_string().await.expect("problem body");
    assert!(!body.contains("<html"), "{body}");
}
