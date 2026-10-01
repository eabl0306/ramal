//! Latencia medida de una comprobacion.

use std::time::Duration;

/// Cuanto tardo en responder un componente.
///
/// La mide el caso de uso con `Instant`, que es un reloj monotono: medir una
/// duracion no es leer la hora de pared, y por eso no pasa por el puerto `Clock`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Latency(Duration);

impl Latency {
    /// Latencia cero, el valor neutro.
    pub const ZERO: Self = Self(Duration::ZERO);

    /// Construye la latencia a partir de una duracion medida.
    #[must_use]
    pub const fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    /// Construye la latencia a partir de milisegundos enteros.
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    /// Latencia en milisegundos, saturando en [`u64::MAX`] en vez de truncar.
    #[must_use]
    pub fn as_millis(&self) -> u64 {
        u64::try_from(self.0.as_millis()).unwrap_or(u64::MAX)
    }

    /// Duracion subyacente.
    #[must_use]
    pub const fn as_duration(&self) -> Duration {
        self.0
    }
}

impl From<Duration> for Latency {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Latency;

    #[test]
    fn converts_a_duration_to_whole_milliseconds() {
        let latency = Latency::from_duration(Duration::from_micros(1_500));
        assert_eq!(latency.as_millis(), 1);
    }

    #[test]
    fn zero_is_the_neutral_value() {
        assert_eq!(Latency::ZERO.as_millis(), 0);
        assert_eq!(Latency::default(), Latency::ZERO);
    }

    #[test]
    fn saturates_instead_of_truncating() {
        let latency = Latency::from_duration(Duration::MAX);
        assert_eq!(latency.as_millis(), u64::MAX);
    }

    #[test]
    fn keeps_the_original_duration() {
        let latency = Latency::from(Duration::from_millis(42));
        assert_eq!(latency.as_duration(), Duration::from_millis(42));
        assert_eq!(latency, Latency::from_millis(42));
    }
}
