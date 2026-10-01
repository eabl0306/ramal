//! Seccion `database`: como llegar a PostgreSQL.

use serde::{Deserialize, Serialize};

/// Configuracion de PostgreSQL (`RAMAL__DATABASE__*`).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct DatabaseConfig {
    /// Cadena de conexion completa, con credenciales.
    pub url: String,
    /// Conexiones maximas del pool.
    pub max_connections: u32,
    /// Tiempo maximo, en milisegundos, para conseguir una conexion del pool.
    pub acquire_timeout_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://ramal:ramal@localhost:5432/ramal".to_owned(),
            max_connections: 5,
            acquire_timeout_ms: 3_000,
        }
    }
}

/// La URL lleva la contrasena: un `{:?}` de la configuracion en un log no puede
/// sacarla.
impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DatabaseConfig")
            .field("url", &"<redacted>")
            .field("max_connections", &self.max_connections)
            .field("acquire_timeout_ms", &self.acquire_timeout_ms)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::DatabaseConfig;

    #[test]
    fn debug_never_prints_the_connection_string() {
        let config = DatabaseConfig {
            url: "postgres://user:hunter2@db/x".to_owned(),
            ..DatabaseConfig::default()
        };
        let rendered = format!("{config:?}");
        assert!(!rendered.contains("hunter2"));
        assert!(rendered.contains("<redacted>"));
        assert!(rendered.contains("max_connections: 5"));
    }
}
