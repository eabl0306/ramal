//! `graph/window`.

use ramal_core_domain::RepoId;
use ramal_protocol::RpcError;
use ramal_protocol::methods::GraphWindowMethod;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::error::{engine_error, invalid_params};
use crate::mapping::graph::window_to_dto;

pub(super) async fn window(
    cases: &UseCases,
    params: serde_json::Value,
) -> Result<serde_json::Value, RpcError> {
    let params = parse_params::<GraphWindowMethod>(params)?;
    let offset = usize::try_from(params.offset).map_err(|_| invalid_params("offset too large"))?;
    let limit = usize::try_from(params.limit).map_err(|_| invalid_params("limit too large"))?;
    let repo_id = RepoId::new(params.repo_id);
    let window = cases
        .graph
        .execute(&repo_id, offset, limit, params.snapshot.as_deref())
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&window_to_dto(&repo_id, &window))
}
