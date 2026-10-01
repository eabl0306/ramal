//! Casos de uso, un modulo por contexto.
//!
//! Cada caso de uso vive en `use_case/<contexto>/<verbo_nombre>.rs`, es un struct
//! con un metodo `execute` y guarda sus puertos como `Arc<dyn Puerto>`: el
//! adaptador HTTP no admite handlers genericos sobre el tipo del puerto (ver el
//! ADR 0003). Los contextos planificados anaden aqui su modulo.

pub mod health;
