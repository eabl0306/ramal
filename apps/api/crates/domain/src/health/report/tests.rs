#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use chrono::{DateTime, Utc};

use super::{HealthReport, HealthStatus};
use crate::health::ComponentHealth;
use crate::value_object::{ComponentName, Latency, ServiceVersion};

fn name(raw: &str) -> ComponentName {
    ComponentName::parse(raw).expect("fixture name should be valid")
}

fn version() -> ServiceVersion {
    ServiceVersion::new(0, 1, 0)
}

fn instant() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-09-30T10:00:00Z")
        .expect("fixture instant should be valid")
        .with_timezone(&Utc)
}

#[test]
fn a_report_without_components_is_healthy() {
    let report = HealthReport::without_components(version(), instant());
    assert_eq!(report.status(), HealthStatus::Ok);
    assert!(report.is_healthy());
    assert!(report.components().is_empty());
    assert_eq!(report.version(), version());
    assert_eq!(report.checked_at(), instant());
}

#[test]
fn all_components_ok_keep_the_report_healthy() {
    let report = HealthReport::new(
        version(),
        instant(),
        vec![
            ComponentHealth::healthy(name("database"), Latency::from_millis(3)),
            ComponentHealth::healthy(name("secrets"), Latency::from_millis(1)),
        ],
    );
    assert_eq!(report.status(), HealthStatus::Ok);
    assert!(report.failing_components().is_empty());
}

#[test]
fn a_single_failing_component_degrades_the_whole_report() {
    let report = HealthReport::new(
        version(),
        instant(),
        vec![
            ComponentHealth::healthy(name("secrets"), Latency::from_millis(1)),
            ComponentHealth::failed(name("database"), Latency::from_millis(50), "timeout"),
        ],
    );
    assert_eq!(report.status(), HealthStatus::Degraded);
    assert!(!report.is_healthy());
    let failing = report.failing_components();
    assert_eq!(failing.len(), 1);
    assert_eq!(failing[0].detail(), Some("timeout"));
    let names: Vec<&str> = report.failing_names().iter().map(|n| n.as_str()).collect();
    assert_eq!(names, vec!["database"]);
}

#[test]
fn status_renders_the_public_contract_values() {
    assert_eq!(HealthStatus::Ok.to_string(), "ok");
    assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
    assert!(HealthStatus::Ok.is_ok());
    assert!(!HealthStatus::Degraded.is_ok());
}
