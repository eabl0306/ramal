//! Stashes y su contenido.
//!
//! Aplicar un stash a ciegas es la forma habitual de liarla, asi que la interfaz
//! ensena antes que ficheros trae y su diff, los no rastreados incluidos. Un stash
//! es un commit con dos o tres padres: el HEAD de cuando se hizo, el indice y, si
//! se guardaron los no rastreados, un tercero con ellos. Los stashes no son
//! historia del proyecto y no salen en el grafo.

use crate::diff::FileDiff;
use crate::sha::Sha;
use crate::time::GitTime;

/// Un stash de la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StashEntry {
    /// Posicion en la lista: `stash@{index}`.
    pub index: u32,
    /// Commit del stash.
    pub sha: Sha,
    /// Mensaje que escribio el usuario, o el asunto del commit en el que se hizo.
    pub message: String,
    /// Rama en la que se hizo; `None` si HEAD estaba separado.
    pub branch: Option<String>,
    /// Cuando se hizo.
    pub created_at: GitTime,
}

/// Lo que trae un stash.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StashContents {
    /// Cambios en ficheros rastreados, contra el commit en el que se hizo.
    pub tracked: Vec<FileDiff>,
    /// Ficheros sin seguimiento que se guardaron con `--include-untracked`.
    pub untracked: Vec<FileDiff>,
}

/// Rama y mensaje a partir del asunto del reflog del stash.
///
/// git escribe `WIP on <rama>: <sha> <asunto>` si no se dio mensaje y
/// `On <rama>: <mensaje>` si se dio. Sin rama, `(no branch)`.
#[must_use]
pub fn parse_stash_subject(subject: &str) -> (Option<String>, String) {
    let rest = subject
        .strip_prefix("WIP on ")
        .or_else(|| subject.strip_prefix("On "));
    let Some((branch, message)) = rest.and_then(|rest| rest.split_once(": ")) else {
        return (None, subject.to_owned());
    };
    let branch = (branch != "(no branch)").then(|| branch.to_owned());
    let message = if subject.starts_with("WIP on ") {
        // `<sha> <asunto>`: el sha corto no le dice nada a quien lee la lista.
        message
            .split_once(' ')
            .map_or(message, |(_, summary)| summary)
    } else {
        message
    };
    (branch, message.to_owned())
}

#[cfg(test)]
mod tests;
