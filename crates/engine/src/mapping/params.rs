//! Parametros del protocolo -> tipos del dominio.
//!
//! Se valida lo que llega de un webview que no es de fiar: shas completos, rutas
//! relativas sin `..`, nombres de ref completos y fechas ISO 8601 estrictas. Lo
//! que no encaja es un error `invalid_params`, antes de llegar al caso de uso.

use git_rudder_core_application::DiffRequest;
use git_rudder_core_domain::diff::{DiffOptions, DiffTarget, Whitespace};
use git_rudder_core_domain::graph::GraphFilter;
use git_rudder_core_domain::{GitTime, RefName, RepoPath, Sha};
use git_rudder_protocol::RpcError;
use git_rudder_protocol::diff as diff_dto;
use git_rudder_protocol::graph as graph_dto;

use super::error::invalid_params;

/// Un sha completo.
///
/// # Errors
///
/// `invalid_params` si no son 40 cifras hexadecimales.
pub fn sha(text: &str) -> Result<Sha, RpcError> {
    Sha::parse(text).map_err(|error| invalid_params(&error.to_string()))
}

/// Una ruta relativa al repositorio.
///
/// # Errors
///
/// `invalid_params` si es absoluta o puede salir del arbol de trabajo.
pub fn path(text: &str) -> Result<RepoPath, RpcError> {
    RepoPath::parse(text).map_err(|error| invalid_params(&error.to_string()))
}

/// Un entero del protocolo como `usize`.
///
/// # Errors
///
/// `invalid_params` si no cabe (no pasa en las plataformas que se soportan).
pub fn count(value: u32, name: &str) -> Result<usize, RpcError> {
    usize::try_from(value).map_err(|_| invalid_params(&format!("{name} too large")))
}

/// El diff pedido.
///
/// # Errors
///
/// `invalid_params` si un sha o una ruta no son validos.
pub fn diff_request(params: &diff_dto::DiffFileParams) -> Result<DiffRequest, RpcError> {
    let target = match &params.target {
        diff_dto::DiffTarget::WorkingTree => DiffTarget::WorkingTree,
        diff_dto::DiffTarget::Index => DiffTarget::Index,
        diff_dto::DiffTarget::Commit { sha: commit } => DiffTarget::Commit(sha(commit)?),
        diff_dto::DiffTarget::Range { from, to } => DiffTarget::Range {
            from: sha(from)?,
            to: sha(to)?,
        },
    };
    let options = params.options.unwrap_or_default();
    let defaults = DiffOptions::default();
    Ok(DiffRequest {
        target,
        path: path(&params.path)?,
        old_path: params.old_path.as_deref().map(path).transpose()?,
        options: DiffOptions {
            whitespace: options.whitespace.map_or(defaults.whitespace, whitespace),
            word_diff: options.word_diff.unwrap_or(defaults.word_diff),
            context_lines: options.context_lines.unwrap_or(defaults.context_lines),
        },
    })
}

const fn whitespace(value: diff_dto::Whitespace) -> Whitespace {
    match value {
        diff_dto::Whitespace::Exact => Whitespace::Exact,
        diff_dto::Whitespace::IgnoreAll => Whitespace::IgnoreAll,
        diff_dto::Whitespace::IgnoreChange => Whitespace::IgnoreChange,
        diff_dto::Whitespace::IgnoreAtEol => Whitespace::IgnoreAtEol,
    }
}

/// El filtro del grafo.
///
/// # Errors
///
/// `invalid_params` si una ref no es un nombre completo o una fecha no es ISO
/// 8601 estricta.
pub fn graph_filter(filter: Option<&graph_dto::GraphFilter>) -> Result<GraphFilter, RpcError> {
    let Some(filter) = filter else {
        return Ok(GraphFilter::default());
    };
    let refs = |names: &Option<Vec<String>>| -> Result<Vec<RefName>, RpcError> {
        names
            .iter()
            .flatten()
            .map(|name| RefName::parse(name).map_err(|error| invalid_params(&error.to_string())))
            .collect()
    };
    let instant = |text: &Option<String>| -> Result<Option<i64>, RpcError> {
        text.as_deref()
            .map(|text| {
                GitTime::parse_iso8601(text)
                    .map(|time| time.seconds())
                    .map_err(|error| invalid_params(&error.to_string()))
            })
            .transpose()
    };
    Ok(GraphFilter {
        hidden: refs(&filter.hidden)?,
        only: refs(&filter.only)?,
        author: filter
            .author
            .as_deref()
            .map(str::trim)
            .filter(|author| !author.is_empty())
            .map(str::to_owned),
        since: instant(&filter.since)?,
        until: instant(&filter.until)?,
    })
}

#[cfg(test)]
mod tests;
