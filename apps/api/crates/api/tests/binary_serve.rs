//! El binario de punta a punta: escucha, responde las sondas y para con SIGTERM.
//!
//! El caso con base de datos se salta sin `RAMAL_TEST_DATABASE_URL` y falla sin
//! ella si `RAMAL_TEST_REQUIRE_DATABASE=1`; vive aqui y no en los tests de
//! `ramal-api-http` porque necesita los adaptadores reales, y el adaptador HTTP
//! no puede depender de la infraestructura ni en sus tests.

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
#![cfg(unix)]

mod support;

use std::time::Duration;

use support::http_client::{get, get_until};
use support::process::{ApiProcess, UNREACHABLE_DATABASE};

#[test]
fn without_a_database_the_api_is_alive_but_not_ready() {
    let api = ApiProcess::start(UNREACHABLE_DATABASE);

    let live = get(api.port(), "/health/live").expect("liveness answers");
    assert_eq!(live.status, 200);
    assert!(live.body.contains(r#""status":"ok""#), "{}", live.body);
    assert!(live.headers.contains("x-request-id:"));

    let ready = get(api.port(), "/health/ready").expect("readiness answers");
    assert_eq!(ready.status, 503);
    assert!(
        ready.body.contains(r#""status":"degraded""#),
        "{}",
        ready.body
    );
    assert!(
        ready.body.contains(r#""name":"database""#),
        "{}",
        ready.body
    );

    let missing = get(api.port(), "/v1/ruta-que-no-existe").expect("the api answers");
    assert_eq!(missing.status, 404);
    assert!(
        missing
            .headers
            .contains("content-type: application/problem+json")
    );

    // Sin RAMAL__SERVER__SWAGGER_UI, como arranca la imagen de produccion.
    let swagger = get(api.port(), "/swagger-ui/").expect("the api answers");
    assert_eq!(swagger.status, 404);

    assert!(
        api.terminate().success(),
        "SIGTERM should end with exit code 0"
    );
}

#[test]
fn swagger_ui_is_served_only_when_the_environment_asks_for_it() {
    let api = ApiProcess::start_with(
        UNREACHABLE_DATABASE,
        &[("RAMAL__SERVER__SWAGGER_UI", "true")],
    );

    let swagger = get(api.port(), "/swagger-ui/").expect("the api answers");
    assert_eq!(swagger.status, 200);
    assert!(swagger.headers.contains("content-type: text/html"));

    assert!(api.terminate().success());
}

#[test]
fn with_a_database_the_api_migrates_and_becomes_ready() {
    let Some(url) = support::database_url() else {
        return;
    };
    let api = ApiProcess::start(&url);

    let ready = get_until(api.port(), "/health/ready", 200, Duration::from_secs(30));
    assert_eq!(ready.status, 200, "{}", ready.body);
    assert!(ready.body.contains(r#""status":"ok""#), "{}", ready.body);
    assert!(
        ready.body.contains(r#""name":"database""#),
        "{}",
        ready.body
    );

    assert!(
        api.terminate().success(),
        "SIGTERM should end with exit code 0"
    );
}
