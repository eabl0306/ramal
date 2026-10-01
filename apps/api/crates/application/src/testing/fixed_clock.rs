//! Reloj de test que no avanza solo.

use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};

use crate::port::Clock;

/// Reloj con la hora puesta a mano.
#[derive(Debug)]
pub struct FixedClock {
    now: Mutex<DateTime<Utc>>,
}

impl FixedClock {
    /// Reloj parado en el instante dado.
    #[must_use]
    pub const fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    /// Reloj parado en [`FixedClock::fixture_instant`].
    #[must_use]
    pub fn at_fixture() -> Self {
        Self::new(Self::fixture_instant())
    }

    /// Instante fijo y conocido que usan los tests: 2026-09-30T10:00:00Z.
    #[must_use]
    pub fn fixture_instant() -> DateTime<Utc> {
        let Ok(parsed) = DateTime::parse_from_rfc3339("2026-09-30T10:00:00Z") else {
            unreachable!("the fixture instant is a valid rfc3339 literal")
        };
        parsed.with_timezone(&Utc)
    }

    /// Mueve el reloj hacia delante.
    pub fn advance(&self, delta: Duration) {
        if let Ok(mut now) = self.now.lock() {
            *now += delta;
        }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
            .lock()
            .map_or_else(|poisoned| *poisoned.into_inner(), |now| *now)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::FixedClock;
    use crate::port::Clock;

    #[test]
    fn stays_still_until_someone_moves_it() {
        let clock = FixedClock::at_fixture();
        assert_eq!(clock.now(), FixedClock::fixture_instant());
        clock.advance(Duration::minutes(15));
        assert_eq!(
            clock.now(),
            FixedClock::fixture_instant() + Duration::minutes(15)
        );
    }
}
