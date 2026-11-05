//! Tabla metodo -> handler, partida por area.
//!
//! Es una tabla y no un `match`: con veinte metodos, un `match` superaria la
//! ciclomatica de 10 (ADR 0005), y una tabla se recorre igual con uno que con
//! cien. Un test comprueba que cada metodo de `define_methods!` tiene su fila.

mod engine;
mod graph;
mod reads;
mod repo;

use std::future::Future;
use std::pin::Pin;

use git_rudder_protocol::methods::{
    BlameFileMethod, CommitDetailMethod, CompareRefsMethod, ConflictReadMethod, DiffFileMethod,
    EngineInfoMethod, GitConfigureMethod, GraphSearchMethod, GraphWindowMethod, HistoryFileMethod,
    RefsListMethod, RepoOpenMethod, RepoTrustMethod, ReposScanMethod, ReposSummaryMethod,
    StashListMethod, StashShowMethod, StatusReadMethod, SubmoduleListMethod, SubmoduleOpenMethod,
};
use git_rudder_protocol::{MethodInfo, RpcError, RpcMethod};
use serde::Serialize;
use serde_json::Value;

use crate::engine::{CallContext, CallOrigin, UseCases};
use crate::mapping::error::{forbidden_method, internal_error, invalid_params, method_not_found};

/// Futuro de un handler.
type Reply<'a> = Pin<Box<dyn Future<Output = Result<Value, RpcError>> + Send + 'a>>;

/// Un handler: casos de uso y parametros en JSON.
type Handler = for<'a> fn(&'a UseCases, Value) -> Reply<'a>;

/// Cada metodo con su handler.
pub const HANDLERS: &[(&str, Handler)] = &[
    (EngineInfoMethod::NAME, |c, p| Box::pin(engine::info(c, p))),
    (GitConfigureMethod::NAME, |c, p| {
        Box::pin(engine::configure(c, p))
    }),
    (RepoOpenMethod::NAME, |c, p| Box::pin(repo::open(c, p))),
    (RepoTrustMethod::NAME, |c, p| Box::pin(repo::trust(c, p))),
    (ReposSummaryMethod::NAME, |c, p| {
        Box::pin(repo::summary(c, p))
    }),
    (ReposScanMethod::NAME, |c, p| Box::pin(repo::scan(c, p))),
    (SubmoduleOpenMethod::NAME, |c, p| {
        Box::pin(repo::open_submodule(c, p))
    }),
    (GraphWindowMethod::NAME, |c, p| {
        Box::pin(graph::window(c, p))
    }),
    (GraphSearchMethod::NAME, |c, p| {
        Box::pin(graph::search(c, p))
    }),
    (StatusReadMethod::NAME, |c, p| Box::pin(reads::status(c, p))),
    (DiffFileMethod::NAME, |c, p| Box::pin(reads::diff(c, p))),
    (CommitDetailMethod::NAME, |c, p| {
        Box::pin(reads::commit(c, p))
    }),
    (RefsListMethod::NAME, |c, p| Box::pin(reads::refs(c, p))),
    (StashListMethod::NAME, |c, p| Box::pin(reads::stashes(c, p))),
    (StashShowMethod::NAME, |c, p| Box::pin(reads::stash(c, p))),
    (SubmoduleListMethod::NAME, |c, p| {
        Box::pin(reads::submodules(c, p))
    }),
    (HistoryFileMethod::NAME, |c, p| {
        Box::pin(reads::history(c, p))
    }),
    (BlameFileMethod::NAME, |c, p| Box::pin(reads::blame(c, p))),
    (CompareRefsMethod::NAME, |c, p| {
        Box::pin(reads::compare(c, p))
    }),
    (ConflictReadMethod::NAME, |c, p| {
        Box::pin(reads::conflict(c, p))
    }),
];

/// Encamina una llamada a su handler.
pub async fn route(
    cases: &UseCases,
    method: &str,
    params: Value,
    context: CallContext,
) -> Result<Value, RpcError> {
    let Some(info) = MethodInfo::find(method) else {
        return Err(method_not_found(method));
    };
    if info.host_only && context.origin == CallOrigin::Webview {
        tracing::warn!(method, "host-only method called from a webview");
        return Err(forbidden_method(method));
    }
    let handler = HANDLERS
        .iter()
        .find(|(name, _)| *name == info.name)
        .map(|(_, handler)| *handler)
        .ok_or_else(|| method_not_found(method))?;
    handler(cases, params).await
}

/// Parametros del metodo `M`. Tienen que ser un objeto: serde aceptaria tambien
/// un array posicional, y el protocolo no lo admite.
fn parse_params<M: RpcMethod>(params: Value) -> Result<M::Params, RpcError> {
    if !params.is_object() {
        return Err(invalid_params("params must be an object"));
    }
    serde_json::from_value(params).map_err(|error| invalid_params(&error.to_string()))
}

/// Resultado serializado.
fn to_result<T: Serialize>(result: &T) -> Result<Value, RpcError> {
    serde_json::to_value(result).map_err(|error| internal_error(&error.to_string()))
}

#[cfg(test)]
mod tests;
