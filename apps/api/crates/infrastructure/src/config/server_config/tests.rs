#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{ConfigError, ServerConfig};

#[test]
fn the_default_listens_only_on_loopback() {
    let address = ServerConfig::default()
        .socket_addr()
        .expect("the default should resolve");
    assert_eq!(address.to_string(), "127.0.0.1:8080");
}

#[test]
fn swagger_ui_is_off_by_default() {
    assert!(!ServerConfig::default().swagger_ui);
}

#[test]
fn rejects_a_host_that_is_not_an_ip() {
    let server = ServerConfig {
        host: "not an ip".to_owned(),
        ..ServerConfig::default()
    };
    let error = server.socket_addr().expect_err("a name is not an ip");
    assert!(matches!(error, ConfigError::InvalidAddress { .. }));
    assert_eq!(error.to_string(), "invalid listen address not an ip:8080");
}
