//! `repo/open`: abrir un repositorio por ruta. Solo lo llaman los hosts.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Parametros de `repo/open`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct OpenRepoParams {
    /// Carpeta del repositorio o cualquier carpeta dentro de el.
    pub path: String,
}

/// Repositorio abierto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RepoHandle {
    /// Id opaco con el que se pide todo lo demas.
    pub repo_id: String,
    /// Raiz del arbol de trabajo, para ensenarla (nunca vuelve como parametro).
    pub root: String,
    /// Nombre corto de la rama de HEAD (`main`); falta con HEAD separado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub head_ref: Option<String>,
    /// Commit de HEAD; falta si la rama todavia no tiene commits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub head_sha: Option<String>,
    /// HEAD apunta a un commit y no a una rama.
    pub is_detached: bool,
    /// El repositorio no tiene ningun commit.
    pub is_empty: bool,
}
