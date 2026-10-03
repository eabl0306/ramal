//! Adaptador driven: el reloj del sistema.

use chrono::{DateTime, Utc};
use git_rudder_api_application::Clock;

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
mod tests;
