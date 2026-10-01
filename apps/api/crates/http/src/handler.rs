//! Handlers, un modulo por contexto.
//!
//! Cada handler es un struct con sus casos de uso detras de `Arc` y un bloque
//! `impl` con `#[endpoint]` y un metodo `handle`. Nada de `impl` genericos: la
//! macro de salvo-oapi no los admite (ADR 0003).

mod health;

pub use health::{LivenessHandler, ReadinessHandler};
