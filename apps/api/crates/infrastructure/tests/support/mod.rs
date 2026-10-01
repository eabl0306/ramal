//! Acceso a un PostgreSQL de verdad para los tests de integracion.
//!
//! Sin `RAMAL_TEST_DATABASE_URL` los tests se saltan solos, para que `cargo test`
//! pase en una maquina sin base de datos. Con `RAMAL_TEST_REQUIRE_DATABASE=1`
//! saltarse deja de estar permitido y la falta de la URL es un fallo: lo ponen
//! `just api-test-db` y la CI, porque un test que se salta en silencio en la CI
//! es un verde falso.

#![allow(
    dead_code,
    reason = "each integration test binary compiles this module and uses only the part it needs"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail loudly on purpose; a panic here is the assertion"
)]

use ramal_api_infrastructure::{DatabaseConfig, PgPool, connect_pool, run_migrations};

/// Variable que habilita los tests contra PostgreSQL.
pub const URL_VAR: &str = "RAMAL_TEST_DATABASE_URL";

/// Variable que convierte el salto en fallo.
pub const REQUIRE_VAR: &str = "RAMAL_TEST_REQUIRE_DATABASE";

/// Indica si la base es obligatoria. La bandera es explicita a proposito: solo
/// el comando que promete probar la base la enciende.
pub fn database_is_mandatory() -> bool {
    std::env::var(REQUIRE_VAR)
        .is_ok_and(|value| !matches!(value.trim(), "" | "0" | "false" | "FALSE"))
}

/// URL de la base de pruebas, o `None` si se pueden saltar.
pub fn database_url() -> Option<String> {
    let url = std::env::var(URL_VAR).ok().filter(|url| !url.is_empty());
    assert!(
        url.is_some() || !database_is_mandatory(),
        "{URL_VAR} is empty but {REQUIRE_VAR} is set: \
         these tests must not pass without a real postgres"
    );
    url
}

/// Configuracion de pool para los tests.
pub fn test_config(url: String) -> DatabaseConfig {
    DatabaseConfig {
        url,
        max_connections: 2,
        // Los tests en paralelo hacen cola en el advisory lock del migrador.
        acquire_timeout_ms: 30_000,
    }
}

/// Pool sobre la base de pruebas con las migraciones aplicadas.
pub async fn migrated_pool() -> Option<PgPool> {
    let url = database_url()?;
    let pool = connect_pool(&test_config(url)).expect("pool should build");
    run_migrations(&pool)
        .await
        .expect("migrations should apply");
    Some(pool)
}

/// Sufijo unico para nombres que crea un test: la base de desarrollo acumula
/// filas y esquemas de otras ejecuciones, y ningun test asume que esta vacia.
pub fn unique_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after 1970")
        .as_nanos();
    format!("{}_{nanos}", std::process::id())
}
