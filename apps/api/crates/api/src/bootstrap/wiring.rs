//! Piezas compartidas que el bootstrap reparte entre contextos.

use std::sync::Arc;

use ramal_api_application::Clock;
use ramal_api_domain::ServiceVersion;
use ramal_api_infrastructure::{AppConfig, PgPool, SystemClock, connect_pool};

use crate::error::StartupError;

/// Lo que todos los contextos comparten: version, pool y reloj.
///
/// Se construye una vez por proceso. Cada `bootstrap/<contexto>.rs` lee de aqui
/// lo que necesita y construye sus adaptadores y casos de uso. El primer
/// contexto con seccion propia de configuracion la anade aqui.
pub struct Wiring {
    /// Version del servicio.
    pub version: ServiceVersion,
    /// Pool perezoso de PostgreSQL.
    pub pool: PgPool,
    /// Reloj del sistema, detras del puerto.
    pub clock: Arc<dyn Clock>,
}

impl Wiring {
    /// Crea el pool (sin conectar todavia) y el reloj.
    ///
    /// # Errors
    ///
    /// Devuelve [`StartupError::Persistence`] si la cadena de conexion no vale.
    pub fn new(config: &AppConfig, version: ServiceVersion) -> Result<Self, StartupError> {
        let pool = connect_pool(&config.database)?;
        Ok(Self {
            version,
            pool,
            clock: Arc::new(SystemClock::new()),
        })
    }
}
