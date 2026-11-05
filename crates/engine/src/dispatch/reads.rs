//! Lecturas de un repositorio abierto: estado, diff, commit, refs, stashes,
//! submodulos, historia de un fichero, blame, comparar y conflictos.

use git_rudder_core_domain::RepoId;
use git_rudder_protocol::RpcError;
use git_rudder_protocol::methods::{
    BlameFileMethod, CommitDetailMethod, CompareRefsMethod, ConflictReadMethod, DiffFileMethod,
    HistoryFileMethod, RefsListMethod, StashListMethod, StashShowMethod, StatusReadMethod,
    SubmoduleListMethod,
};
use serde_json::Value;

use super::{parse_params, to_result};
use crate::engine::UseCases;
use crate::mapping::diff::{detail_to_dto, diff_to_dto};
use crate::mapping::error::engine_error;
use crate::mapping::inventory::{
    blame_to_dto, comparison_to_dto, conflict_to_file_dto, history_to_dto, listing_to_dto,
    stash_contents_to_dto, stashes_to_dto, submodules_to_dto,
};
use crate::mapping::params::{count, diff_request, path, sha};
use crate::mapping::status::status_to_dto;

pub(super) async fn status(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let repo_id = RepoId::new(parse_params::<StatusReadMethod>(params)?.repo_id);
    let status = cases
        .status
        .execute(&repo_id)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&status_to_dto(&repo_id, &status))
}

pub(super) async fn diff(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<DiffFileMethod>(params)?;
    let request = diff_request(&params)?;
    let diff = cases
        .diff
        .execute(&RepoId::new(params.repo_id), request)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&diff_to_dto(&diff))
}

pub(super) async fn commit(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<CommitDetailMethod>(params)?;
    let commit = sha(&params.sha)?;
    let detail = cases
        .commit
        .execute(&RepoId::new(params.repo_id), commit)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&detail_to_dto(&detail))
}

pub(super) async fn refs(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let repo_id = RepoId::new(parse_params::<RefsListMethod>(params)?.repo_id);
    let listing = cases
        .refs
        .execute(&repo_id)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&listing_to_dto(&repo_id, &listing))
}

pub(super) async fn stashes(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let repo_id = RepoId::new(parse_params::<StashListMethod>(params)?.repo_id);
    let stashes = cases
        .stashes
        .execute(&repo_id)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&stashes_to_dto(&stashes))
}

pub(super) async fn stash(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<StashShowMethod>(params)?;
    let stash = sha(&params.sha)?;
    let contents = cases
        .stash
        .execute(&RepoId::new(params.repo_id), stash)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&stash_contents_to_dto(&contents))
}

pub(super) async fn submodules(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let repo_id = RepoId::new(parse_params::<SubmoduleListMethod>(params)?.repo_id);
    let submodules = cases
        .submodules
        .execute(&repo_id)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&submodules_to_dto(&submodules))
}

pub(super) async fn history(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<HistoryFileMethod>(params)?;
    let file = path(&params.path)?;
    let page = cases
        .history
        .execute(
            &RepoId::new(params.repo_id),
            &file,
            count(params.offset, "offset")?,
            count(params.limit, "limit")?,
        )
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&history_to_dto(&page))
}

pub(super) async fn blame(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<BlameFileMethod>(params)?;
    let file = path(&params.path)?;
    let revision = params.revision.as_deref().map(sha).transpose()?;
    let blame = cases
        .blame
        .execute(&RepoId::new(params.repo_id), &file, revision)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&blame_to_dto(&blame))
}

pub(super) async fn compare(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<CompareRefsMethod>(params)?;
    let comparison = cases
        .compare
        .execute(&RepoId::new(params.repo_id), &params.base, &params.head)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&comparison_to_dto(&comparison))
}

pub(super) async fn conflict(cases: &UseCases, params: Value) -> Result<Value, RpcError> {
    let params = parse_params::<ConflictReadMethod>(params)?;
    let file = path(&params.path)?;
    let conflict = cases
        .conflict
        .execute(&RepoId::new(params.repo_id), &file)
        .await
        .map_err(|e| engine_error(&e))?;
    to_result(&conflict_to_file_dto(&conflict))
}
