//! Commits: el resumen que comparten la busqueda, la historia de un fichero y la
//! comparacion de ramas, y el detalle de `commit/detail`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::diff::FileStatus;

/// Lo que se ensena de un commit en una lista.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitSummary {
    /// Sha completo.
    pub sha: String,
    /// Sha corto que dio git.
    pub short_sha: String,
    /// Padres en orden.
    pub parents: Vec<String>,
    /// Primera linea del mensaje.
    pub summary: String,
    /// Nombre del autor.
    pub author_name: String,
    /// Correo del autor.
    pub author_email: String,
    /// Fecha de autor, ISO 8601.
    pub authored_at: String,
}

/// Un fichero cambiado en un commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangedFile {
    /// Ruta.
    pub path: String,
    /// Ruta anterior si se renombro o se copio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub old_path: Option<String>,
    /// Que le paso.
    pub status: FileStatus,
    /// Lineas anadidas; falta si es binario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub additions: Option<u32>,
    /// Lineas quitadas; falta si es binario.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub deletions: Option<u32>,
}

/// Parametros de `commit/detail`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct CommitDetailParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Sha completo.
    pub sha: String,
}

/// Resultado de `commit/detail`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitDetail {
    /// Sha completo, para copiarlo.
    pub sha: String,
    /// Sha corto.
    pub short_sha: String,
    /// Padres en orden; un merge los lista todos.
    pub parents: Vec<String>,
    /// Nombre del autor.
    pub author_name: String,
    /// Correo del autor.
    pub author_email: String,
    /// Fecha de autor, ISO 8601.
    pub authored_at: String,
    /// Nombre de quien hizo el commit.
    pub committer_name: String,
    /// Correo de quien hizo el commit.
    pub committer_email: String,
    /// Fecha del commit, ISO 8601.
    pub committed_at: String,
    /// Primera linea del mensaje.
    pub summary: String,
    /// Mensaje completo.
    pub message: String,
    /// Ficheros cambiados contra el primer padre.
    pub files: Vec<ChangedFile>,
}
