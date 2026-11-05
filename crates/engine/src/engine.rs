//! `Engine`: la puerta de los dos hosts. `handle` para un `RpcRequest` entero,
//! `dispatch` para un metodo con sus parametros.

use git_rudder_core_application::{
    BlameFile, CompareRefs, ConfigureGit, DescribeEngine, FileHistory, ListRefs, ListStashes,
    ListSubmodules, LoadGraphWindow, OpenRepository, OpenSubmodule, ReadCommitDetail, ReadConflict,
    ReadFileDiff, ReadStatus, ScanRepositories, SearchHistory, ShowStash, SummarizeRepository,
    TrustRepository,
};
use git_rudder_protocol::{RequestId, RpcError, RpcRequest, RpcResponse};
use serde::Deserialize;

use crate::cancel::Cancellations;
use crate::dispatch;
use crate::mapping::error::cancelled;

/// Notificacion de control con la que la interfaz cancela una peticion.
pub const CANCEL_NOTIFICATION: &str = "$/cancel";

/// Quien hace la llamada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallOrigin {
    /// El propio host (el comando `repo_pick_and_open`, el extension host).
    #[default]
    Host,
    /// Un webview, que pinta contenido no confiable: no puede llamar a los metodos
    /// marcados como del host en `define_methods!`.
    Webview,
}

/// Contexto de una llamada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CallContext {
    /// Quien llama.
    pub origin: CallOrigin,
}

impl CallContext {
    /// Llamada del host.
    #[must_use]
    pub const fn host() -> Self {
        Self {
            origin: CallOrigin::Host,
        }
    }

    /// Llamada reenviada desde un webview. Los hosts ya filtran los metodos del
    /// host; el motor lo vuelve a comprobar porque es la autoridad.
    #[must_use]
    pub const fn webview() -> Self {
        Self {
            origin: CallOrigin::Webview,
        }
    }
}

/// Casos de uso cableados por el builder.
pub struct UseCases {
    pub describe: DescribeEngine,
    pub configure_git: ConfigureGit,
    pub open: std::sync::Arc<OpenRepository>,
    pub trust: TrustRepository,
    pub open_submodule: OpenSubmodule,
    pub summarize: SummarizeRepository,
    pub scan: ScanRepositories,
    pub graph: LoadGraphWindow,
    pub search: SearchHistory,
    pub status: ReadStatus,
    pub diff: ReadFileDiff,
    pub commit: ReadCommitDetail,
    pub refs: ListRefs,
    pub stashes: ListStashes,
    pub stash: ShowStash,
    pub submodules: ListSubmodules,
    pub history: FileHistory,
    pub blame: BlameFile,
    pub compare: CompareRefs,
    pub conflict: ReadConflict,
}

/// Parametros de `$/cancel`.
#[derive(Debug, Deserialize)]
struct CancelParams {
    id: RequestId,
}

/// El motor del cliente.
pub struct Engine {
    use_cases: UseCases,
    cancellations: Cancellations,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Engine").finish_non_exhaustive()
    }
}

impl Engine {
    pub(crate) fn new(use_cases: UseCases) -> Self {
        Self {
            use_cases,
            cancellations: Cancellations::default(),
        }
    }

    /// Punto de entrada de los dos hosts para peticiones del propio host.
    /// `None` para notificaciones (sin id): no llevan respuesta. La unica que
    /// atiende el motor es `$/cancel`.
    pub async fn handle(&self, request: RpcRequest) -> Option<RpcResponse> {
        self.handle_with(request, CallContext::host()).await
    }

    /// Como [`Engine::handle`], indicando quien llama. Una peticion con id se
    /// puede cancelar con `$/cancel` mientras esta en curso: responde entonces
    /// `cancelled` y el git que estuviera esperando se mata.
    pub async fn handle_with(
        &self,
        request: RpcRequest,
        context: CallContext,
    ) -> Option<RpcResponse> {
        let Some(id) = request.id else {
            self.notification(&request);
            return None;
        };
        let call = self.dispatch(&request.method, request.params, context);
        let response = match self.cancellations.track(id, call).await {
            Some(Ok(result)) => RpcResponse::success(Some(id), result),
            Some(Err(error)) => RpcResponse::failure(Some(id), error),
            None => RpcResponse::failure(Some(id), cancelled()),
        };
        Some(response)
    }

    /// Llama al metodo con sus parametros. Lo usa `handle` por dentro y los tests.
    ///
    /// # Errors
    ///
    /// [`RpcError`] con el `kind` estable si el metodo no existe, los parametros no
    /// encajan o el caso de uso falla.
    pub async fn dispatch(
        &self,
        method: &str,
        params: serde_json::Value,
        context: CallContext,
    ) -> Result<serde_json::Value, RpcError> {
        dispatch::route(&self.use_cases, method, params, context).await
    }

    fn notification(&self, request: &RpcRequest) {
        match cancel_target(request) {
            Ok(id) => {
                self.cancellations.cancel(id);
            }
            Err(reason) => {
                tracing::debug!(method = %request.method, reason, "notification ignored");
            }
        }
    }
}

/// La peticion que una notificacion `$/cancel` pide cancelar, o por que se ignora.
fn cancel_target(request: &RpcRequest) -> Result<RequestId, &'static str> {
    if request.method != CANCEL_NOTIFICATION {
        return Err("unknown notification");
    }
    serde_json::from_value::<CancelParams>(request.params.clone())
        .map(|params| params.id)
        .map_err(|_| "malformed $/cancel")
}
