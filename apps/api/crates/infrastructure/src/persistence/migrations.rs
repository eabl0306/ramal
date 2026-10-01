//! Migraciones embebidas en el binario.
//!
//! Los ficheros viven en `apps/api/migrations`, solo hacia delante. Una
//! migracion fusionada en `main` no se edita nunca: SQLx guarda su suma y se
//! niega a seguir si el fichero cambio, que es la herramienta haciendo cumplir la
//! regla.

use sqlx::PgPool;
use sqlx::migrate::Migrator;

use super::pool::PersistenceError;

/// Migraciones de `apps/api/migrations`, metidas en el binario al compilar.
static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// Aplica las migraciones pendientes.
///
/// SQLx toma un advisory lock, asi que dos procesos que arrancan a la vez no se
/// pisan: uno espera al otro.
///
/// # Errors
///
/// Devuelve [`PersistenceError::Migrate`] si alguna migracion falla o si la base
/// no responde.
pub async fn run_migrations(pool: &PgPool) -> Result<(), PersistenceError> {
    tracing::info!(
        migrations = MIGRATOR.iter().len(),
        "applying database migrations"
    );
    MIGRATOR.run(pool).await.map_err(PersistenceError::Migrate)
}

/// Version de la ultima migracion que trae este binario.
///
/// La sonda de readiness compara la base con este numero: un esquema por detras
/// significa que las migraciones del arranque fallaron.
#[must_use]
pub fn latest_migration_version() -> i64 {
    MIGRATOR
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{MIGRATOR, latest_migration_version};

    #[test]
    fn the_migration_set_is_embedded_and_not_empty() {
        assert!(MIGRATOR.iter().len() > 0);
    }

    #[test]
    fn the_latest_version_is_the_newest_file() {
        let newest = MIGRATOR.iter().map(|m| m.version).max();
        assert_eq!(Some(latest_migration_version()), newest);
        assert!(latest_migration_version() >= 20_261_001_000_000);
    }

    #[test]
    fn migrations_are_forward_only() {
        assert!(
            MIGRATOR
                .iter()
                .all(|migration| !migration.migration_type.is_down_migration()),
            "apps/api/migrations only takes forward migrations (ADR 0004)"
        );
    }
}
