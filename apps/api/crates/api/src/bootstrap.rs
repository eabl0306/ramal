//! Montaje de la aplicacion: el unico sitio que conoce los adaptadores concretos.
//!
//! Cada contexto tiene su modulo en `bootstrap/<contexto>.rs` con una funcion
//! `dependencies(&Wiring)`: asi dos contextos que se cablean a la vez no se pisan
//! en este fichero. Para anadir uno, se rellena su modulo y su linea en
//! [`router_dependencies`].

mod health;
mod migrations;
mod wiring;

use std::path::Path;

use ramal_api_domain::ServiceVersion;
use ramal_api_http::RouterDependencies;
use ramal_api_infrastructure::AppConfig;

use crate::error::StartupError;
use crate::shutdown;
use wiring::Wiring;

/// Version del servicio, leida del manifiesto del paquete.
///
/// # Errors
///
/// Devuelve [`StartupError::Version`] si la version no es `major.minor.patch`.
pub fn service_version() -> Result<ServiceVersion, StartupError> {
    ServiceVersion::parse(env!("CARGO_PKG_VERSION")).map_err(StartupError::from)
}

/// Lanza las migraciones y atiende hasta recibir SIGINT o SIGTERM.
///
/// # Errors
///
/// Devuelve [`StartupError`] si la direccion, el pool o el puerto fallan. Que la
/// base no responda no es un error de arranque: lo cuenta `/health/ready`.
pub async fn serve(config: AppConfig) -> Result<(), StartupError> {
    let version = service_version()?;
    let address = config.socket_addr()?;
    let wiring = Wiring::new(&config, version)?;
    migrations::spawn(wiring.pool.clone());
    let swagger_ui = config.server.swagger_ui;
    tracing::info!(%version, swagger_ui, "ramal api is starting");
    ramal_api_http::serve(
        router_dependencies(&wiring, &config),
        address,
        shutdown::signal(),
        shutdown::GRACE_PERIOD,
    )
    .await?;
    Ok(())
}

/// Escribe el documento OpenAPI en `path` sin tocar la base de datos.
///
/// Se cablea el router de verdad, el mismo que arranca [`serve`], para que el
/// documento salga de las rutas reales. El pool es perezoso y nadie lo usa, asi
/// que no se abre ninguna conexion; eso si, SQLx necesita un runtime de tokio
/// para crearlo. La configuracion es la de por defecto: el documento no depende
/// del entorno y `just api-openapi` debe dar el mismo resultado en cualquier
/// maquina.
///
/// # Errors
///
/// Devuelve [`StartupError`] si el documento no se puede construir o escribir.
pub fn write_openapi(path: &Path) -> Result<(), StartupError> {
    let config = AppConfig::default();
    let wiring = Wiring::new(&config, service_version()?)?;
    let document = ramal_api_http::document_json(&router_dependencies(&wiring, &config))
        .map_err(|error| StartupError::OpenApi(error.to_string()))?;
    std::fs::write(path, document).map_err(|source| StartupError::WriteOpenApi {
        path: path.to_path_buf(),
        source,
    })?;
    tracing::info!(path = %path.display(), "openapi document written");
    Ok(())
}

/// Reune las dependencias de todos los contextos y lo que la configuracion
/// decide del router.
fn router_dependencies(wiring: &Wiring, config: &AppConfig) -> RouterDependencies {
    RouterDependencies {
        version: wiring.version,
        health: health::dependencies(wiring),
        swagger_ui: config.server.swagger_ui,
    }
}
