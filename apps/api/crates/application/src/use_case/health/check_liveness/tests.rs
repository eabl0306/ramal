use std::sync::Arc;

use git_rudder_api_domain::{HealthStatus, ServiceVersion};

use super::CheckLiveness;
use crate::testing::FixedClock;

#[test]
fn liveness_is_always_healthy_and_has_no_components() {
    let clock = Arc::new(FixedClock::at_fixture());
    let report = CheckLiveness::new(ServiceVersion::new(1, 2, 3), clock).execute();
    assert_eq!(report.status(), HealthStatus::Ok);
    assert!(report.components().is_empty(), "{:?}", report.components());
    assert_eq!(report.version(), ServiceVersion::new(1, 2, 3));
    assert_eq!(report.checked_at(), FixedClock::fixture_instant());
}

#[test]
fn debug_shows_the_version_and_hides_the_clock() {
    let use_case = CheckLiveness::new(
        ServiceVersion::new(0, 1, 0),
        Arc::new(FixedClock::at_fixture()),
    );
    let rendered = format!("{use_case:?}");
    assert!(rendered.contains("CheckLiveness"));
    assert!(rendered.contains("version"));
}
