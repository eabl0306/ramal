//! Adaptador driven: el reloj del sistema.

use chrono::{DateTime, Utc};
use ramal_api_application::Clock;

/// Reloj que devuelve la hora real en UTC.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    /// Crea el reloj.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use ramal_api_application::Clock;

    use super::SystemClock;

    #[test]
    fn reads_a_time_after_the_fixture_instant() {
        let Ok(fixture) = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z") else {
            unreachable!("the literal is a valid instant")
        };
        assert!(SystemClock::new().now() > fixture.with_timezone(&Utc));
    }
}
