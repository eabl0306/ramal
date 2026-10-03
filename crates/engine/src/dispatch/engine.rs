//! `engine/info`.

use git_rudder_protocol::RpcError;
use git_rudder_protocol::methods::EngineInfoMethod;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::engine::description_to_dto;
use crate::mapping::error::engine_error;

pub(super) async fn info(
    cases: &UseCases,
    params: serde_json::Value,
) -> Result<serde_json::Value, RpcError> {
    parse_params::<EngineInfoMethod>(params)?;
    let description = cases
        .describe
        .execute()
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&description_to_dto(&description))
}
