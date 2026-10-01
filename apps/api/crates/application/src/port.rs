//! Puertos que declara el nucleo y que implementan los adaptadores driven.
//!
//! Cada puerto nace con su primer caso de uso: no se declaran puertos que nadie
//! usa. Los contextos planificados anaden aqui su modulo (`port/auth.rs`,
//! `port/vault.rs`...) con sus repositorios y servicios.

mod clock;
mod health_probe;

pub use clock::Clock;
pub use health_probe::{HealthProbe, ProbeError};
