//! Mapeo dominio -> DTO y `EngineError` -> `RpcError`.
//!
//! Son funciones libres y no `impl From`: ni el tipo de dominio ni el DTO son de
//! este crate, y la regla de huerfanos de Rust no deja implementar `From` entre
//! dos tipos ajenos.

pub mod engine;
pub mod error;
pub mod graph;
pub mod repo;

/// Los DTOs no llevan enteros de 64 bits (ts-rs los haria `bigint`). Filas y
/// carriles caben de sobra en 32 bits; si algun dia no cupieran, se satura en vez
/// de dar la vuelta.
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
