//! Casos de uso del contexto `health`.

mod check_liveness;
mod check_readiness;

pub use check_liveness::CheckLiveness;
pub use check_readiness::{CheckReadiness, Probes};
