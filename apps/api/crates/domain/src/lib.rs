//! Dominio de la API de Ramal: valores y reglas puras.
//!
//! Este crate no conoce HTTP, SQL ni ningun runtime asincrono. Sus unicas
//! dependencias son librerias igual de puras (`thiserror` y `chrono`), y el test
//! de arquitectura de `ramal-api` lo comprueba en cada build. Su `clippy.toml`
//! prohibe ademas procesos, ficheros, red, entorno y reloj de pared.
//!
//! El primer corte solo trae el contexto `health`. Cada contexto planificado
//! (`auth`, `devices`, `organizations`, `workspaces`, `repositories`, `settings`,
//! `vault`, `linked_accounts`, `org_ai`, `platform_oauth`, `releases`) anade aqui
//! su modulo con la misma forma: `contexto.rs` mas la carpeta `contexto/`.

pub mod health;
pub mod value_object;

pub use health::{ComponentHealth, ComponentStatus, HealthReport, HealthStatus};
pub use value_object::{
    ComponentName, ComponentNameError, Latency, ServiceVersion, ServiceVersionError,
};
