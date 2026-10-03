//! Migraciones al arrancar, sin tumbar el proceso si fallan.

use std::time::Duration;

use git_rudder_api_infrastructure::{PersistenceError, PgPool, run_migrations};

/// Espera entre reintentos cuando las migraciones fallan.
const RETRY_INTERVAL: Duration = Duration::from_secs(10);

/// Lanza las migraciones en segundo plano y vuelve enseguida.
///
/// Si fallan, el proceso sigue vivo y `/health/ready` responde 503, como en
/// Gastonomo: un contenedor que muere porque la base tarda entra en un bucle de
/// reinicios. La sonda de la base compara el esquema con la ultima migracion del
/// binario, asi que el servicio no se declara listo hasta que terminan bien.
///
/// Dos diferencias con Gastonomo, las dos para que una base lenta no cueste un
/// reinicio: el servidor escucha mientras tanto (liveness responde desde el
/// primer segundo) y un fallo se reintenta cada [`RETRY_INTERVAL`] hasta que
/// sale bien.
pub fn spawn(pool: PgPool) {
    tokio::spawn(apply_until_done(pool));
}

async fn apply_until_done(pool: PgPool) {
    while let Err(error) = run_migrations(&pool).await {
        log::failed(&error);
        tokio::time::sleep(RETRY_INTERVAL).await;
    }
    log::applied();
}

/// Trazas de las migraciones. Cada macro de `tracing` expande a bastante codigo
/// y, mezclada con la logica, dispara el limite de complejidad cognitiva.
mod log {
    use super::{PersistenceError, RETRY_INTERVAL};

    pub fn applied() {
        tracing::info!("database migrations are up to date");
    }

    pub fn failed(error: &PersistenceError) {
        tracing::warn!(
            %error,
            retry_in_secs = RETRY_INTERVAL.as_secs(),
            "could not apply migrations; /health/ready answers 503 until they succeed"
        );
    }
}
