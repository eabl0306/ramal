//! Adaptador driven: comprueba PostgreSQL a traves del puerto `HealthProbe`.

use std::time::Duration;

use async_trait::async_trait;
use ramal_api_application::{HealthProbe, ProbeError};
use ramal_api_domain::ComponentName;
use sqlx::PgPool;
use tokio::time::timeout;

/// Nombre publico del componente, fijado por el contrato de `/health/ready`.
const COMPONENT: &str = "database";

/// Codigo de PostgreSQL para "la tabla no existe".
const UNDEFINED_TABLE: &str = "42P01";

/// Ultima migracion aplicada con exito, o 0 si no hay ninguna.
const APPLIED_VERSION: &str =
    "SELECT COALESCE(MAX(version), 0)::bigint FROM _sqlx_migrations WHERE success";

/// Comprueba que PostgreSQL responde y que su esquema esta al dia.
///
/// Un `SELECT 1` no basta: si las migraciones del arranque fallaron, la base
/// contesta pero el esquema no es el que espera el codigo, y el servicio no debe
/// recibir trafico. Por eso la consulta lee la ultima migracion aplicada y la
/// compara con la mas nueva que trae el binario. Un esquema por delante si vale:
/// cada migracion es compatible con el binario anterior (ADR 0004), y durante un
/// despliegue conviven los dos.
#[derive(Debug, Clone)]
pub struct PostgresHealthProbe {
    name: ComponentName,
    pool: PgPool,
    timeout: Duration,
    expected_version: i64,
}

impl PostgresHealthProbe {
    /// Crea la sonda sobre un pool ya construido.
    #[must_use]
    pub fn new(pool: PgPool, probe_timeout: Duration, expected_version: i64) -> Self {
        let name = match ComponentName::parse(COMPONENT) {
            Ok(name) => name,
            Err(error) => unreachable!("component name constant is invalid: {error}"),
        };
        Self {
            name,
            pool,
            timeout: probe_timeout,
            expected_version,
        }
    }

    fn compare(&self, applied: i64) -> Result<(), ProbeError> {
        if applied >= self.expected_version {
            return Ok(());
        }
        Err(ProbeError::new(format!(
            "database schema is at migration {applied}, this build expects {}",
            self.expected_version
        )))
    }
}

#[async_trait]
impl HealthProbe for PostgresHealthProbe {
    fn name(&self) -> &ComponentName {
        &self.name
    }

    async fn check(&self) -> Result<(), ProbeError> {
        let query = sqlx::query_scalar::<_, i64>(APPLIED_VERSION).fetch_one(&self.pool);
        match timeout(self.timeout, query).await {
            Ok(Ok(applied)) => self.compare(applied),
            Ok(Err(error)) => Err(ProbeError::new(describe(&error))),
            Err(_) => Err(ProbeError::new(format!(
                "postgres did not answer within {}ms",
                self.timeout.as_millis()
            ))),
        }
    }
}

fn describe(error: &sqlx::Error) -> String {
    let undefined_table = error
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| code == UNDEFINED_TABLE);
    if undefined_table {
        "migrations have not been applied yet".to_owned()
    } else {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail loudly on purpose; a panic here is the assertion"
    )]

    use std::time::Duration;

    use ramal_api_application::HealthProbe;

    use super::{PostgresHealthProbe, describe};
    use crate::config::DatabaseConfig;
    use crate::persistence::connect_pool;

    fn unreachable_probe(millis: u64) -> PostgresHealthProbe {
        let config = DatabaseConfig {
            // Nadie escucha en el puerto 1: la conexion se rechaza al momento.
            url: "postgres://u:p@127.0.0.1:1/none".to_owned(),
            ..DatabaseConfig::default()
        };
        let pool = connect_pool(&config).expect("lazy pool should build");
        PostgresHealthProbe::new(pool, Duration::from_millis(millis), 1)
    }

    #[tokio::test]
    async fn exposes_the_component_name_of_the_contract() {
        assert_eq!(unreachable_probe(50).name().as_str(), "database");
    }

    #[tokio::test]
    async fn reports_an_error_when_postgres_is_unreachable() {
        let error = unreachable_probe(300)
            .check()
            .await
            .expect_err("port 1 should refuse");
        assert!(!error.message().is_empty());
    }

    #[tokio::test]
    async fn a_schema_behind_the_build_is_not_ready() {
        let probe = unreachable_probe(50);
        let error = probe.compare(0).expect_err("0 is behind 1");
        assert_eq!(
            error.message(),
            "database schema is at migration 0, this build expects 1"
        );
        assert!(probe.compare(1).is_ok());
        assert!(
            probe.compare(2).is_ok(),
            "a newer schema is still compatible"
        );
    }

    #[test]
    fn errors_that_are_not_from_the_database_keep_their_text() {
        let message = describe(&sqlx::Error::PoolTimedOut);
        assert!(message.contains("timed out"));
    }
}
