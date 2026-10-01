#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use super::{ComponentHealth, ComponentStatus};
use crate::value_object::{ComponentName, Latency};

fn database() -> ComponentName {
    ComponentName::parse("database").expect("fixture name should be valid")
}

#[test]
fn a_healthy_reading_has_no_detail() {
    let reading = ComponentHealth::healthy(database(), Latency::from_millis(7));
    assert_eq!(reading.status(), ComponentStatus::Ok);
    assert_eq!(reading.latency().as_millis(), 7);
    assert_eq!(reading.detail(), None);
    assert_eq!(reading.name().as_str(), "database");
}

#[test]
fn a_failed_reading_keeps_the_reason() {
    let reading =
        ComponentHealth::failed(database(), Latency::from_millis(30), "connection refused");
    assert_eq!(reading.status(), ComponentStatus::Error);
    assert_eq!(reading.detail(), Some("connection refused"));
}

#[test]
fn status_renders_the_public_contract_values() {
    assert_eq!(ComponentStatus::Ok.to_string(), "ok");
    assert_eq!(ComponentStatus::Error.to_string(), "error");
    assert!(ComponentStatus::Ok.is_ok());
    assert!(!ComponentStatus::Error.is_ok());
}
