//! Tabla metodo -> handler, partida por area para que no crezca un `match`
//! gigante (clippy no mide la ciclomatica; ADR 0005).

mod engine;
mod graph;
mod repo;

use git_rudder_protocol::methods::{EngineInfoMethod, GraphWindowMethod, RepoOpenMethod};
use git_rudder_protocol::{MethodInfo, RpcError, RpcMethod};
use serde::Serialize;

use crate::engine::{CallContext, CallOrigin, UseCases};
use crate::mapping::error::{forbidden_method, internal_error, invalid_params, method_not_found};

/// Encamina una llamada a su handler.
pub async fn route(
    cases: &UseCases,
    method: &str,
    params: serde_json::Value,
    context: CallContext,
) -> Result<serde_json::Value, RpcError> {
    let Some(info) = MethodInfo::find(method) else {
        return Err(method_not_found(method));
    };
    if info.host_only && context.origin == CallOrigin::Webview {
        tracing::warn!(method, "host-only method called from a webview");
        return Err(forbidden_method(method));
    }
    match info.name {
        EngineInfoMethod::NAME => engine::info(cases, params).await,
        RepoOpenMethod::NAME => repo::open(cases, params).await,
        GraphWindowMethod::NAME => graph::window(cases, params).await,
        _ => Err(method_not_found(method)),
    }
}

/// Parametros del metodo `M`. Tienen que ser un objeto: serde aceptaria tambien
/// un array posicional, y el protocolo no lo admite.
fn parse_params<M: RpcMethod>(params: serde_json::Value) -> Result<M::Params, RpcError> {
    if !params.is_object() {
        return Err(invalid_params("params must be an object"));
    }
    serde_json::from_value(params).map_err(|error| invalid_params(&error.to_string()))
}

/// Resultado serializado.
fn to_result<T: Serialize>(result: &T) -> Result<serde_json::Value, RpcError> {
    serde_json::to_value(result).map_err(|error| internal_error(&error.to_string()))
}
