#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{TelemetryError, build_filter, init_tracing};
use crate::config::TelemetryConfig;

#[test]
fn accepts_a_plain_level() {
    assert!(build_filter("info").is_ok());
}

#[test]
fn accepts_a_per_target_directive() {
    assert!(build_filter("warn,ramal_api_http=debug").is_ok());
}

#[test]
fn rejects_a_malformed_directive() {
    let error = build_filter("ramal_api_http=not-a-level").expect_err("must fail");
    assert!(error.to_string().contains("invalid log filter"));
}

#[test]
fn a_second_installation_is_reported_instead_of_panicking() {
    let config = TelemetryConfig::default();
    // El primero puede ganar o perder la carrera con otro test del binario;
    // el segundo siempre encuentra uno instalado.
    let _ = init_tracing(&config);
    assert!(matches!(
        init_tracing(&config),
        Err(TelemetryError::AlreadyInitialized)
    ));
}

#[test]
fn an_invalid_level_never_installs_anything() {
    let config = TelemetryConfig {
        log_level: "=".to_owned(),
    };
    assert!(matches!(
        init_tracing(&config),
        Err(TelemetryError::InvalidFilter { .. })
    ));
}
