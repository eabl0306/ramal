//! DTOs del contrato publico, un modulo por contexto.
//!
//! JSON en camelCase, enumerados en snake_case e instantes en RFC 3339 con `Z`
//! (api-surface.md). Cada DTO deriva `ToSchema` con un nombre explicito: ese
//! nombre es el que ve `@ramal/api-client`, y no debe depender de en que modulo
//! viva el tipo.

mod health;

pub use health::{CheckState, HealthCheck, HealthResponse, HealthState};
