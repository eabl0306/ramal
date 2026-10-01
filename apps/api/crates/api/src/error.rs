//! Errores de arranque del servicio.

use std::path::PathBuf;

use ramal_api_domain::ServiceVersionError;
use ramal_api_http::ServeError;
use ramal_api_infrastructure::{ConfigError, PersistenceError};
use thiserror::Error;

/// Motivos por los que el binario no llega a hacer su trabajo.
#[derive(Debug, Error)]
pub enum StartupError {
    /// La configuracion del entorno no es valida.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// La version del paquete no es `major.minor.patch`.
    #[error("invalid package version: {0}")]
    Version(#[from] ServiceVersionError),
    /// El pool de PostgreSQL no se pudo configurar.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    /// El servidor no pudo escuchar.
    #[error(transparent)]
    Serve(#[from] ServeError),
    /// El documento OpenAPI no se pudo serializar.
    #[error("could not build the OpenAPI document: {0}")]
    OpenApi(String),
    /// El documento OpenAPI no se pudo escribir.
    #[error("could not write the OpenAPI document to {path}: {source}")]
    WriteOpenApi {
        /// Ruta de destino.
        path: PathBuf,
        /// Error de entrada y salida.
        #[source]
        source: std::io::Error,
    },
    /// No se pudo crear el runtime asincrono.
    #[error("could not start the async runtime: {0}")]
    Runtime(#[source] std::io::Error),
}
