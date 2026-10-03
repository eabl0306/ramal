//! `Engine`: la puerta de los dos hosts. `handle` para un `RpcRequest` entero,
//! `dispatch` para un metodo con sus parametros.

use git_rudder_core_application::{DescribeEngine, LoadGraphWindow, OpenRepository};
use git_rudder_protocol::{RpcError, RpcRequest, RpcResponse};

use crate::dispatch;

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
    pub open: OpenRepository,
    pub graph: LoadGraphWindow,
}

/// El motor del cliente.
pub struct Engine {
    use_cases: UseCases,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Engine").finish_non_exhaustive()
    }
}

impl Engine {
    pub(crate) const fn new(use_cases: UseCases) -> Self {
        Self { use_cases }
    }

    /// Punto de entrada de los dos hosts para peticiones del propio host.
    /// `None` para notificaciones (sin id): no llevan respuesta y el primer corte
    /// no atiende ninguna (`$/cancel` esta planificado).
    pub async fn handle(&self, request: RpcRequest) -> Option<RpcResponse> {
        self.handle_with(request, CallContext::host()).await
    }

    /// Como [`Engine::handle`], indicando quien llama.
    pub async fn handle_with(
        &self,
        request: RpcRequest,
        context: CallContext,
    ) -> Option<RpcResponse> {
        let Some(id) = request.id else {
            tracing::debug!(method = %request.method, "notification ignored");
            return None;
        };
        let response = match self
            .dispatch(&request.method, request.params, context)
            .await
        {
            Ok(result) => RpcResponse::success(Some(id), result),
            Err(error) => RpcResponse::failure(Some(id), error),
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
}
