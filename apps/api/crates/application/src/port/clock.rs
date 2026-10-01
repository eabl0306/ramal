//! Puerto de reloj.

use chrono::{DateTime, Utc};

/// Da la hora de pared al nucleo.
///
/// Existe para que ningun caso de uso llame a `Utc::now()` (el `clippy.toml` del
/// crate lo prohibe): con un reloj fijo un test comprueba instantes y caducidades
/// sin esperar. El primer uso es el instante de cada informe de salud.
pub trait Clock: Send + Sync {
    /// Instante actual en UTC.
    fn now(&self) -> DateTime<Utc>;
}
