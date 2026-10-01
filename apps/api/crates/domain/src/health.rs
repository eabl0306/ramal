//! Contexto `health`: si el servicio esta vivo y si puede recibir trafico.

mod component_health;
mod report;

pub use component_health::{ComponentHealth, ComponentStatus};
pub use report::{HealthReport, HealthStatus};
