//! `graph/window` y `graph/search`.

use git_rudder_core_application::WindowRequest;
use git_rudder_core_domain::RepoId;
use git_rudder_protocol::RpcError;
use git_rudder_protocol::methods::{GraphSearchMethod, GraphWindowMethod};
use serde_json::Value;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::error::engine_error;
use crate::mapping::graph::window_to_dto;
use crate::mapping::inventory::search_to_dto;
use crate::mapping::params::{count, graph_filter};

/// Resultados de una busqueda si no se piden otros.
const DEFAULT_SEARCH_LIMIT: u32 = 100;

pub(super) async fn window(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<GraphWindowMethod>(params)?;
    let request = WindowRequest {
        offset: count(params.offset, "offset")?,
        limit: count(params.limit, "limit")?,
        snapshot: params.snapshot,
        filter: graph_filter(params.filter.as_ref())?,
    };
    let repo_id = RepoId::new(params.repo_id);
    let window = cases
        .graph
        .execute(&repo_id, &request)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&window_to_dto(&repo_id, &window))
}

pub(super) async fn search(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<GraphSearchMethod>(params)?;
    let filter = graph_filter(params.filter.as_ref())?;
    let limit = count(params.limit.unwrap_or(DEFAULT_SEARCH_LIMIT), "limit")?;
    let repo_id = RepoId::new(params.repo_id);
    let hits = cases
        .search
        .execute(&repo_id, &params.query, limit, &filter)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&search_to_dto(&repo_id, &hits))
}
