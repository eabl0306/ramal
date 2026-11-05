//! Metodos del host sobre repositorios: abrir, confiar, abrir un submodulo, el
//! estado de fondo y buscar repositorios en carpetas.

use std::path::Path;

use git_rudder_core_domain::RepoId;
use git_rudder_protocol::RpcError;
use git_rudder_protocol::methods::{
    RepoOpenMethod, RepoTrustMethod, ReposScanMethod, ReposSummaryMethod, SubmoduleOpenMethod,
};
use git_rudder_protocol::repos::{RepoTrusted, ScanReposResult};
use serde_json::Value;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::error::engine_error;
use crate::mapping::params::{count, path};
use crate::mapping::repo::opened_to_dto;
use crate::mapping::status::summary_to_dto;

/// Profundidad de busqueda si no se pide otra.
const DEFAULT_SCAN_DEPTH: u32 = 3;

pub(super) async fn open(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<RepoOpenMethod>(params)?;
    let opened = cases
        .open
        .execute(Path::new(&params.path), params.trusted.unwrap_or(false))
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&opened_to_dto(&opened))
}

pub(super) async fn trust(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<RepoTrustMethod>(params)?;
    let repo_id = RepoId::new(params.repo_id);
    cases
        .trust
        .execute(&repo_id)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&RepoTrusted {
        repo_id: repo_id.as_str().to_owned(),
        trusted: true,
    })
}

pub(super) async fn open_submodule(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<SubmoduleOpenMethod>(params)?;
    let submodule = path(&params.path)?;
    let opened = cases
        .open_submodule
        .execute(&RepoId::new(params.repo_id), &submodule)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&opened_to_dto(&opened))
}

pub(super) async fn summary(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<ReposSummaryMethod>(params)?;
    let summary = cases
        .summarize
        .execute(Path::new(&params.path))
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&summary_to_dto(&summary))
}

pub(super) async fn scan(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<ReposScanMethod>(params)?;
    let depth = count(params.max_depth.unwrap_or(DEFAULT_SCAN_DEPTH), "maxDepth")?;
    let roots = cases
        .scan
        .execute(Path::new(&params.path), depth)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&ScanReposResult {
        roots: roots
            .iter()
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
    })
}
