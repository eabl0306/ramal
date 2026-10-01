//! Doble del puerto de sonda de salud.

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use ramal_api_domain::ComponentName;

use crate::port::{HealthProbe, ProbeError};

/// Sonda que responde lo que le digas y cuenta cuantas veces la llaman.
#[derive(Debug)]
pub struct FakeHealthProbe {
    name: ComponentName,
    failure: Option<String>,
    calls: AtomicUsize,
}

impl FakeHealthProbe {
    /// Sonda que siempre responde bien.
    ///
    /// Un nombre invalido es un error del propio test y termina el test.
    #[must_use]
    pub fn healthy(name: &str) -> Self {
        Self::build(name, None)
    }

    /// Sonda que siempre falla con el motivo indicado.
    #[must_use]
    pub fn failing(name: &str, reason: &str) -> Self {
        Self::build(name, Some(reason.to_owned()))
    }

    /// Veces que se ha llamado a [`HealthProbe::check`].
    #[must_use]
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }

    fn build(name: &str, failure: Option<String>) -> Self {
        let name = match ComponentName::parse(name) {
            Ok(name) => name,
            Err(error) => unreachable!("invalid fixture component name {name:?}: {error}"),
        };
        Self {
            name,
            failure,
            calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl HealthProbe for FakeHealthProbe {
    fn name(&self) -> &ComponentName {
        &self.name
    }

    async fn check(&self) -> Result<(), ProbeError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.failure
            .as_ref()
            .map_or(Ok(()), |reason| Err(ProbeError::new(reason.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::FakeHealthProbe;
    use crate::port::HealthProbe;
    use crate::testing::block_on;

    #[test]
    fn a_healthy_fake_answers_ok_and_counts_calls() {
        let probe = FakeHealthProbe::healthy("database");
        assert!(block_on(probe.check()).is_ok());
        assert_eq!(probe.calls(), 1);
        assert_eq!(probe.name().as_str(), "database");
    }

    #[test]
    fn a_failing_fake_answers_the_configured_reason() {
        let probe = FakeHealthProbe::failing("database", "boom");
        let outcome = block_on(probe.check()).map_err(|error| error.message().to_owned());
        assert_eq!(outcome, Err("boom".to_owned()));
    }
}
