//! Adaptadores driven de la API de Ramal.
//!
//! Aqui viven las piezas que hablan con el mundo exterior: la configuracion por
//! entorno, la telemetria, el reloj del sistema y PostgreSQL. El nucleo nunca
//! importa este crate; es el composition root (`ramal-api`) quien conecta estos
//! adaptadores con los casos de uso. Ninguna consulta SQL sale de aqui.

pub mod clock;
pub mod config;
pub mod persistence;
pub mod telemetry;

pub use clock::SystemClock;
pub use config::{AppConfig, ConfigError, DatabaseConfig, ServerConfig, TelemetryConfig};
pub use persistence::{
    PersistenceError, PgPool, PostgresHealthProbe, connect_pool, latest_migration_version,
    run_migrations,
};
pub use telemetry::{TelemetryError, init_tracing};
