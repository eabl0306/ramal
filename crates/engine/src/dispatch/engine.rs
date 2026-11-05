//! `engine/info` y `git/configure`.

use std::path::PathBuf;

use git_rudder_protocol::RpcError;
use git_rudder_protocol::methods::{EngineInfoMethod, GitConfigureMethod};
use serde_json::Value;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::engine::{description_to_dto, git_to_dto};
use crate::mapping::error::engine_error;

pub(super) async fn info(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    parse_params::<EngineInfoMethod>(params)?;
    let description = cases
        .describe
        .execute()
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&description_to_dto(&description))
}

pub(super) async fn configure(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<GitConfigureMethod>(params)?;
    let git = cases
        .configure_git
        .execute(params.path.map(PathBuf::from))
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&git_to_dto(&git))
}
