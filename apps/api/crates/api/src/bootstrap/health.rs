//! Cableado del contexto `health`.

use std::sync::Arc;
use std::time::Duration;

use git_rudder_api_application::{CheckLiveness, CheckReadiness, HealthProbe};
use git_rudder_api_http::HealthDependencies;
use git_rudder_api_infrastructure::{PostgresHealthProbe, latest_migration_version};

use super::wiring::Wiring;

/// Tiempo maximo de la sonda de PostgreSQL. Por debajo del timeout de 3
/// segundos del healthcheck de compose, para que la respuesta llegue a tiempo.
const DATABASE_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Casos de uso de `/health/live` y `/health/ready`.
///
/// Las sondas van en el orden en que se publican. La de `secrets` (la llave
/// maestra de las llaves de IA de organizacion) se anade aqui cuando exista.
pub fn dependencies(wiring: &Wiring) -> HealthDependencies {
    let database: Arc<dyn HealthProbe> = Arc::new(PostgresHealthProbe::new(
        wiring.pool.clone(),
        DATABASE_PROBE_TIMEOUT,
        latest_migration_version(),
    ));
    HealthDependencies {
        liveness: Arc::new(CheckLiveness::new(wiring.version, wiring.clock.clone())),
        readiness: Arc::new(CheckReadiness::new(
            wiring.version,
            wiring.clock.clone(),
            vec![database],
        )),
    }
}
