use chrono::{DateTime, Utc};
use git_rudder_api_application::Clock;

use super::SystemClock;

#[test]
fn reads_a_time_after_the_fixture_instant() {
    let Ok(fixture) = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z") else {
        unreachable!("the literal is a valid instant")
    };
    assert!(SystemClock::new().now() > fixture.with_timezone(&Utc));
}
