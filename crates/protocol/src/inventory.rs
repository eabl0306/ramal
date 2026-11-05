//! Stashes, submodulos y conflictos: `stash/list`, `stash/show`,
//! `submodule/list`, `submodule/open` y `conflict/read`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::diff::FileDiff;
use crate::status::ConflictCode;

/// Un stash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StashInfo {
    /// `stash@{index}`.
    pub index: u32,
    /// Sha del stash, con el que se pide su contenido.
    pub sha: String,
    /// Mensaje.
    pub message: String,
    /// Rama en la que se hizo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub branch: Option<String>,
    /// Cuando, ISO 8601.
    pub created_at: String,
}

/// Resultado de `stash/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StashList {
    /// Stashes, el mas reciente primero.
    pub stashes: Vec<StashInfo>,
}

/// Parametros de `stash/show`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct StashShowParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Sha del stash.
    pub sha: String,
}

/// Resultado de `stash/show`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StashContents {
    /// Cambios en ficheros rastreados.
    pub tracked: Vec<FileDiff>,
    /// Ficheros sin seguimiento guardados con el stash.
    pub untracked: Vec<FileDiff>,
}

/// Estado de un submodulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SubmoduleState {
    /// Sin clonar.
    Uninitialized,
    /// En el commit registrado.
    UpToDate,
    /// En otro commit.
    Modified,
    /// En conflicto.
    Conflicted,
}

/// Un submodulo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubmoduleInfo {
    /// Ruta relativa a la raiz.
    pub path: String,
    /// Commit en el que esta.
    pub sha: String,
    /// Estado.
    pub state: SubmoduleState,
    /// Descripcion de git, si tiene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub describe: Option<String>,
    /// Se puede abrir como repositorio.
    pub can_open: bool,
}

/// Resultado de `submodule/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubmoduleList {
    /// Submodulos por ruta, anidados incluidos.
    pub submodules: Vec<SubmoduleInfo>,
}

/// Parametros de los metodos sobre un fichero de un repositorio abierto
/// (`submodule/open`, `conflict/read`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct RepoPathParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Ruta relativa a la raiz, con `/`.
    pub path: String,
}

/// Un trozo de un fichero en conflicto, con la misma forma que el
/// `ConflictSegment` de `@git-rudder/domain`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum ConflictSegment {
    /// Lineas iguales.
    Context {
        /// Lineas.
        lines: Vec<String>,
    },
    /// Un bloque en conflicto.
    Hunk {
        /// Nuestro lado.
        ours: Vec<String>,
        /// El ancestro comun (marcadores zdiff3).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        base: Option<Vec<String>>,
        /// Su lado.
        theirs: Vec<String>,
    },
}

/// Etiquetas de los marcadores.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConflictLabels {
    /// Nuestro lado.
    pub ours: String,
    /// Su lado.
    pub theirs: String,
    /// El ancestro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub base: Option<String>,
}

/// Resultado de `conflict/read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConflictFile {
    /// Ruta.
    pub path: String,
    /// Forma del conflicto.
    pub code: ConflictCode,
    /// Algun lado es binario: se elige un lado entero.
    pub binary: bool,
    /// Segmentos si es texto y existen los dos lados.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub segments: Option<Vec<ConflictSegment>>,
    /// Etiquetas del primer bloque.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub labels: Option<ConflictLabels>,
}
