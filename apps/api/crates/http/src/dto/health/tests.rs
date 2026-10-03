#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use git_rudder_api_application::Clock;
use git_rudder_api_application::testing::FixedClock;
use git_rudder_api_domain::{
    ComponentHealth, ComponentName, HealthReport, Latency, ServiceVersion,
};

use super::HealthResponse;

fn name(raw: &str) -> ComponentName {
    ComponentName::parse(raw).expect("fixture name should be valid")
}

fn json(report: &HealthReport) -> serde_json::Value {
    serde_json::to_value(HealthResponse::from(report)).expect("dto should serialize")
}

#[test]
fn maps_a_healthy_report_to_the_public_contract() {
    let report = HealthReport::new(
        ServiceVersion::new(0, 1, 0),
        FixedClock::at_fixture().now(),
        vec![ComponentHealth::healthy(
            name("database"),
            Latency::from_millis(4),
        )],
    );
    let body = json(&report);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], "0.1.0");
    assert_eq!(body["checkedAt"], "2026-09-30T10:00:00Z");
    assert_eq!(body["checks"][0]["name"], "database");
    assert_eq!(body["checks"][0]["status"], "ok");
    assert_eq!(body["checks"][0]["latencyMs"], 4);
}

#[test]
fn a_degraded_report_never_publishes_the_failure_detail() {
    let report = HealthReport::new(
        ServiceVersion::new(1, 0, 0),
        FixedClock::at_fixture().now(),
        vec![ComponentHealth::failed(
            name("database"),
            Latency::from_millis(120),
            "password authentication failed for user git-rudder",
        )],
    );
    let body = json(&report);
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["checks"][0]["status"], "error");
    assert!(body["checks"][0].get("detail").is_none());
    assert!(!body.to_string().contains("password"));
}

#[test]
fn a_report_without_components_serializes_an_empty_array() {
    let report = HealthReport::without_components(
        ServiceVersion::new(2, 3, 4),
        FixedClock::at_fixture().now(),
    );
    let body = json(&report);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["checks"].as_array().map(Vec::len), Some(0));
}
