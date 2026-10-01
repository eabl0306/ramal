//! Adaptadores de persistencia sobre PostgreSQL con SQLx.
//!
//! Consultas en tiempo de ejecucion (`sqlx::query`), sin las macros `query!`:
//! un `cargo build` recien clonado tiene que funcionar sin base de datos (ADR
//! 0004). Cada contexto planificado anade aqui su `postgres_<contexto>.rs` con
//! los repositorios que implementan sus puertos.

mod migrations;
mod pool;
mod postgres_health_probe;

/// Pool de conexiones de SQLx, reexportado para el composition root.
pub use sqlx::PgPool;

pub use migrations::{latest_migration_version, run_migrations};
pub use pool::{PersistenceError, connect_pool};
pub use postgres_health_probe::PostgresHealthProbe;
