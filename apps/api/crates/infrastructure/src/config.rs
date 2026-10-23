//! Configuracion tipada leida del entorno con figment.
//!
//! Prefijo `GIT_RUDDER__` y `__` como separador de nivel: `GIT_RUDDER__DATABASE__URL` es
//! `database.url`. Todo tiene valor por defecto en el codigo, y `.env.example` es
//! la lista de lo que se puede mover. Cada modulo nuevo que necesite
//! configuracion anade aqui su seccion.

mod app_config;
mod database_config;
mod server_config;
mod telemetry_config;

pub use app_config::{AppConfig, ConfigError, ENV_PREFIX};
pub use database_config::DatabaseConfig;
pub use server_config::ServerConfig;
pub use telemetry_config::TelemetryConfig;
