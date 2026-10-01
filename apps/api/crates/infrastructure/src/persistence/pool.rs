//! Creacion del pool de conexiones.

use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use thiserror::Error;

use crate::config::DatabaseConfig;

/// Fallo de la capa de persistencia.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// La cadena de conexion no vale.
    #[error("could not configure the postgres pool: {0}")]
    Connect(#[source] sqlx::Error),
    /// Las migraciones no se pudieron aplicar.
    #[error("could not apply migrations: {0}")]
    Migrate(#[source] sqlx::migrate::MigrateError),
}

/// Crea el pool de conexiones a PostgreSQL.
///
/// El pool es perezoso: no abre ninguna conexion hasta que alguien la pide. Asi
/// el proceso arranca aunque la base todavia no acepte conexiones, y la sonda de
/// readiness refleja esa situacion en vez de matar el contenedor. Tambien es lo
/// que deja a `ramal-api --openapi` cablear el router sin tocar la base.
///
/// # Errors
///
/// Devuelve [`PersistenceError::Connect`] si la cadena de conexion no es valida.
pub fn connect_pool(config: &DatabaseConfig) -> Result<PgPool, PersistenceError> {
    tracing::debug!(
        max_connections = config.max_connections,
        acquire_timeout_ms = config.acquire_timeout_ms,
        "building the postgres connection pool"
    );
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_millis(config.acquire_timeout_ms))
        .connect_lazy(&config.url)
        .map_err(PersistenceError::Connect)
}

#[cfg(test)]
mod tests;
