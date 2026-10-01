//! `repo/open`. Solo para hosts: la ruta nunca viene de un webview.

use std::path::Path;

use ramal_protocol::RpcError;
use ramal_protocol::methods::RepoOpenMethod;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::error::engine_error;
use crate::mapping::repo::opened_to_dto;

pub(super) async fn open(
    cases: &UseCases,
    params: serde_json::Value,
) -> Result<serde_json::Value, RpcError> {
    let params = parse_params::<RepoOpenMethod>(params)?;
    let opened = cases
        .open
        .execute(Path::new(&params.path))
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&opened_to_dto(&opened))
}
