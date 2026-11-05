//! Composition root del host de escritorio: el motor, los plugins, el estado
//! gestionado y los comandos.
//!
//! La composicion del cliente tiene dos niveles. `EngineBuilder` construye lo que
//! es igual en los dos productos (git CLI); este fichero es la raiz de fuera y
//! aporta lo que solo el escritorio sabe construir: el selector de carpetas sobre
//! el dialogo nativo, las notificaciones sobre `Channel` de Tauri, la carpeta de
//! datos donde se recuerda la confianza y, mas adelante, el `SecretStore` sobre el
//! llavero.

use std::sync::Arc;

use std::path::PathBuf;

use git_rudder_engine::{CallContext, ENGINE_VERSION, Engine, EngineBuilder, EngineConfig};
use git_rudder_protocol::RpcMethod as _;
use git_rudder_protocol::methods::EngineInfoMethod;
use tauri::{App, AppHandle, Builder, Manager as _, Runtime};

use crate::commands;
use crate::notifications::NotificationChannels;
use crate::picker::{DialogFolderPicker, FolderPicker};

/// Fichero de la carpeta de datos de la app donde se recuerda en que
/// repositorios confio el usuario (ADR 0020). No se sincroniza.
pub const TRUST_FILE: &str = "trusted-repositories.txt";

/// Variable de entorno con el filtro de log (sintaxis de `tracing`), por defecto
/// `info`. El log va a stderr.
pub const LOG_ENV_VAR: &str = "GIT_RUDDER_LOG";

/// Solo con la feature `e2e`: carpeta que el selector elige sin ensenar el dialogo.
pub const E2E_FOLDER_ENV_VAR: &str = "GIT_RUDDER_E2E_FOLDER";

/// Arranca la app de escritorio y no vuelve hasta que se cierra.
///
/// # Errors
///
/// El error de Tauri si no puede crear la ventana o el runtime.
pub fn run() -> tauri::Result<()> {
    init_tracing();
    let context = context();
    let channels = Arc::new(NotificationChannels::default());
    let trust_file = data_dir(&context.config().identifier).map(|dir| dir.join(TRUST_FILE));
    let engine = Arc::new(build_engine_with(trust_file, Arc::clone(&channels)));
    assemble(tauri::Builder::default(), engine, channels).run(context)
}

/// La carpeta de datos de la app, la misma que `app_data_dir` de Tauri: la de
/// datos del sistema y el identificador. Se calcula antes de crear la app porque
/// el motor se construye antes.
fn data_dir(identifier: &str) -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join(identifier))
}

/// Todo el arranque menos el bucle de eventos.
///
/// Plugins, motor, comandos y el `setup` que registra el selector de carpetas y
/// deja en el log la version del motor. Existe aparte de `run` para que el test
/// de arranque lo construya sobre el runtime simulado y la cobertura llegue al
/// composition root (ADR 0006).
#[must_use]
pub fn assemble<R: Runtime>(
    builder: Builder<R>,
    engine: Arc<Engine>,
    channels: Arc<NotificationChannels>,
) -> Builder<R> {
    configure(with_plugins(builder), engine, channels).setup(|app| {
        setup(app);
        Ok(())
    })
}

/// El motor con la configuracion por defecto, sin notificaciones y con la
/// confianza en memoria: el de los tests de arranque.
#[must_use]
pub fn build_engine() -> Engine {
    EngineBuilder::new(EngineConfig::default()).build()
}

/// El motor del escritorio: notifica por los canales de los webviews y recuerda
/// la confianza en `trust_file`. El binario de git es el del `PATH` hasta que el
/// ajuste lo cambie con `git/configure`.
#[must_use]
pub fn build_engine_with(
    trust_file: Option<PathBuf>,
    channels: Arc<NotificationChannels>,
) -> Engine {
    EngineBuilder::new(EngineConfig {
        trust_file,
        ..EngineConfig::default()
    })
    .with_notification_sink(channels)
    .build()
}

/// El motor y los canales de notificacion como estado gestionado, y los tres
/// comandos. Es lo comun a la app real y a los tests, que lo montan sobre el
/// runtime simulado de Tauri.
#[must_use]
pub fn configure<R: Runtime>(
    builder: Builder<R>,
    engine: Arc<Engine>,
    channels: Arc<NotificationChannels>,
) -> Builder<R> {
    builder
        .manage(engine)
        .manage(channels)
        .invoke_handler(tauri::generate_handler![
            commands::rpc,
            commands::rpc_subscribe,
            commands::repo_pick_and_open
        ])
}

/// La configuracion de `tauri.conf.json`, los recursos del frontend y las
/// capacidades compiladas, para cualquier runtime.
#[must_use]
pub fn context<R: Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

/// Plugins de Tauri solo donde manda el sistema operativo: el dialogo de carpetas
/// (usado solo desde Rust) y, en los builds de E2E, el servidor WebDriver.
fn with_plugins<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let builder = builder.plugin(tauri_plugin_dialog::init());
    #[cfg(feature = "e2e")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    builder
}

fn setup<R: Runtime>(app: &App<R>) {
    app.manage(folder_picker(app.handle()));
    let engine = Arc::clone(app.state::<Arc<Engine>>().inner());
    tauri::async_runtime::spawn(async move { log_engine_info(&engine).await });
}

/// El dialogo nativo; en un build de E2E con la variable puesta, la carpeta fija.
fn folder_picker<R: Runtime>(app: &AppHandle<R>) -> Arc<dyn FolderPicker> {
    #[cfg(feature = "e2e")]
    if let Some(folder) = std::env::var_os(E2E_FOLDER_ENV_VAR) {
        tracing::warn!("e2e build: the folder picker answers with {E2E_FOLDER_ENV_VAR}");
        let folder = std::path::PathBuf::from(folder);
        return Arc::new(crate::picker::FixedFolderPicker::choosing(folder));
    }
    Arc::new(DialogFolderPicker::new(app.clone()))
}

/// Deja en el log la version del motor y la de git al arrancar: es lo primero que
/// se mira cuando algo falla en la maquina de un usuario.
async fn log_engine_info(engine: &Engine) {
    let params = serde_json::Value::Object(serde_json::Map::new());
    match engine
        .dispatch(EngineInfoMethod::NAME, params, CallContext::host())
        .await
    {
        Ok(info) => log_ready(&info),
        Err(error) => log_without_git(&error.message),
    }
}

// Las macros de tracing cuentan para `cognitive_complexity`, asi que cada linea
// de log va en su propia funcion (lo mismo hace el sidecar de VS Code).
fn log_ready(info: &serde_json::Value) {
    tracing::info!(version = ENGINE_VERSION, %info, "engine ready");
}

fn log_without_git(message: &str) {
    tracing::warn!(message, "engine started without a usable git");
}

/// Log a stderr con el filtro de `GIT_RUDDER_LOG`. Si ya hay un subscriber (en los
/// tests), se respeta.
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_env(LOG_ENV_VAR)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();
}

#[cfg(test)]
mod tests;
