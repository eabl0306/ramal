//! Comandos de Tauri: `rpc` y `repo_pick_and_open`. Son adaptadores driving: no
//! hay reglas de negocio aqui, todo lo decide el motor.
//!
//! No hay un comando por metodo del motor: `rpc` lleva el mismo sobre JSON-RPC que
//! el sidecar de VS Code, y la lista de metodos vive en `git-rudder-protocol`. Cada
//! comando necesita un permiso explicito en `capabilities/default.json` porque
//! `build.rs` los declara en el `AppManifest`.

use std::path::Path;
use std::sync::Arc;

use git_rudder_engine::mapping::error::{internal_error, invalid_params};
use git_rudder_engine::{CallContext, Engine};
use git_rudder_protocol::methods::RepoOpenMethod;
use git_rudder_protocol::repo::{OpenRepoParams, RepoHandle};
use git_rudder_protocol::{RpcError, RpcMethod as _, RpcRequest, RpcResponse};
use tauri::{Runtime, State, Window};

use crate::picker::FolderPicker;
use crate::surfaces;

/// Una peticion JSON-RPC del webview al motor.
///
/// Solo pasa si el metodo esta en la lista de la ventana que llama; el motor la
/// recibe marcada como de un webview y vuelve a rechazar los metodos del host,
/// porque es la autoridad.
///
/// Devuelve `None` para las notificaciones (sin id). Un error del motor viaja
/// dentro de la respuesta: `invoke` solo rechaza por fallos de transporte.
///
/// # Errors
///
/// Nunca: Tauri exige un `Result` a los comandos asincronos que toman estado
/// prestado.
#[tauri::command]
pub async fn rpc<R: Runtime>(
    window: Window<R>,
    engine: State<'_, Arc<Engine>>,
    request: RpcRequest,
) -> Result<Option<RpcResponse>, RpcError> {
    Ok(forward(&engine, window.label(), request).await)
}

/// El cuerpo de `rpc`, sin Tauri: la lista de la ventana y despues el motor.
pub async fn forward(engine: &Engine, window: &str, request: RpcRequest) -> Option<RpcResponse> {
    let is_notification = request.id.is_none();
    match surfaces::authorize(window, &request.method, is_notification) {
        Ok(()) => engine.handle_with(request, CallContext::webview()).await,
        Err(error) => reject(window, &request, error),
    }
}

fn reject(window: &str, request: &RpcRequest, error: RpcError) -> Option<RpcResponse> {
    tracing::warn!(window, method = %request.method, "request rejected by the window allowlist");
    request.id.map(|id| RpcResponse::failure(Some(id), error))
}

/// Ensena el dialogo nativo de carpetas y abre en el motor el repositorio elegido.
///
/// El webview nunca manda la ruta: la elige el usuario en el dialogo y solo vuelve
/// el `RepoHandle`. `None` si el usuario cancela.
///
/// # Errors
///
/// El [`RpcError`] del motor si la carpeta no es un repositorio, falta git o git
/// falla. El frontend lo traduce por su `kind`.
#[tauri::command]
pub async fn repo_pick_and_open(
    engine: State<'_, Arc<Engine>>,
    picker: State<'_, Arc<dyn FolderPicker>>,
) -> Result<Option<RepoHandle>, RpcError> {
    let Ok(Some(folder)) = picker.pick_folder().await else {
        return Ok(None);
    };
    open_folder(&engine, &folder).await.map(Some)
}

/// Abre una carpeta con `repo/open` como llamada del host.
///
/// # Errors
///
/// El [`RpcError`] del motor, o uno de parametros si la ruta no es UTF-8 (el
/// protocolo lleva las rutas como texto).
pub async fn open_folder(engine: &Engine, folder: &Path) -> Result<RepoHandle, RpcError> {
    let Some(path) = folder.to_str() else {
        return Err(invalid_params("the folder path is not valid UTF-8"));
    };
    let params = serde_json::to_value(OpenRepoParams {
        path: path.to_owned(),
    })
    .map_err(|error| internal_error(&error.to_string()))?;
    let result = engine
        .dispatch(RepoOpenMethod::NAME, params, CallContext::host())
        .await?;
    serde_json::from_value(result).map_err(|error| internal_error(&error.to_string()))
}
