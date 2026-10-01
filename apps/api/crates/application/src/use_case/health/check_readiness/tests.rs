#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use std::sync::Arc;

use ramal_api_domain::{ComponentStatus, HealthStatus, ServiceVersion};

use super::CheckReadiness;
use crate::port::HealthProbe;
use crate::testing::{FakeHealthProbe, FixedClock, block_on};

fn use_case(probes: Vec<Arc<dyn HealthProbe>>) -> CheckReadiness {
    CheckReadiness::new(
        ServiceVersion::new(0, 1, 0),
        Arc::new(FixedClock::at_fixture()),
        probes,
    )
}

#[test]
fn a_check_without_probes_is_healthy() {
    let report = block_on(use_case(Vec::new()).execute());
    assert_eq!(report.status(), HealthStatus::Ok);
    assert!(report.components().is_empty());
    assert_eq!(report.checked_at(), FixedClock::fixture_instant());
}

#[test]
fn every_probe_produces_one_reading_in_order() {
    let report = block_on(
        use_case(vec![
            Arc::new(FakeHealthProbe::healthy("database")),
            Arc::new(FakeHealthProbe::healthy("secrets")),
        ])
        .execute(),
    );
    assert_eq!(report.status(), HealthStatus::Ok);
    let names: Vec<&str> = report
        .components()
        .iter()
        .map(|component| component.name().as_str())
        .collect();
    assert_eq!(names, vec!["database", "secrets"]);
}

#[test]
fn a_failing_probe_degrades_the_report_and_keeps_the_reason() {
    let report = block_on(
        use_case(vec![
            Arc::new(FakeHealthProbe::healthy("secrets")),
            Arc::new(FakeHealthProbe::failing("database", "connection refused")),
        ])
        .execute(),
    );
    assert_eq!(report.status(), HealthStatus::Degraded);
    let failed = &report.components()[1];
    assert_eq!(failed.status(), ComponentStatus::Error);
    assert_eq!(failed.detail(), Some("connection refused"));
    assert_eq!(failed.name().as_str(), "database");
}

#[test]
fn each_execution_calls_each_probe_once() {
    let probe = Arc::new(FakeHealthProbe::healthy("database"));
    let use_case = use_case(vec![probe.clone()]);
    block_on(use_case.execute());
    block_on(use_case.execute());
    assert_eq!(probe.calls(), 2);
}

#[test]
fn debug_counts_the_probes_without_their_internals() {
    let use_case = use_case(vec![Arc::new(FakeHealthProbe::healthy("database"))]);
    assert!(format!("{use_case:?}").contains("probes: 1"));
}
