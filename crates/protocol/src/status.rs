//! `status/read`: el estado del arbol de trabajo con la rama, los ficheros y la
//! operacion a medias.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Parametros de los metodos que solo necesitan el repositorio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct RepoParams {
    /// Repositorio abierto.
    pub repo_id: String,
}

/// Que le paso a un fichero en un lado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ChangeKind {
    /// Sin cambios en este lado.
    Unmodified,
    /// Contenido cambiado.
    Modified,
    /// Cambio de tipo.
    TypeChanged,
    /// Anadido.
    Added,
    /// Borrado.
    Deleted,
    /// Renombrado.
    Renamed,
    /// Copiado.
    Copied,
}

/// Forma de un conflicto, con las letras de git en el comentario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ConflictCode {
    /// `DD`.
    BothDeleted,
    /// `AU`.
    AddedByUs,
    /// `UD`.
    DeletedByThem,
    /// `UA`.
    AddedByThem,
    /// `DU`.
    DeletedByUs,
    /// `AA`.
    BothAdded,
    /// `UU`.
    BothModified,
}

/// Lo que cambio dentro de un submodulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubmoduleChange {
    /// Apunta a otro commit.
    pub commit_changed: bool,
    /// Tiene ficheros rastreados con cambios.
    pub has_tracked_changes: bool,
    /// Tiene ficheros sin seguimiento.
    pub has_untracked: bool,
}

/// Estado de una entrada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum EntryState {
    /// Rastreado con cambios.
    Changed {
        /// Cambio preparado.
        index: ChangeKind,
        /// Cambio sin preparar.
        worktree: ChangeKind,
        /// Cambios dentro, si es un submodulo.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        submodule: Option<SubmoduleChange>,
    },
    /// En conflicto.
    Unmerged {
        /// Forma del conflicto.
        conflict: ConflictCode,
    },
    /// Sin seguimiento.
    Untracked,
    /// Ignorado.
    Ignored,
}

/// Un fichero con algo que contar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StatusEntry {
    /// Ruta relativa a la raiz, con `/`.
    pub path: String,
    /// Ruta de origen de un renombrado o una copia.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub original_path: Option<String>,
    /// Estado.
    pub state: EntryState,
    /// Tiene algo preparado.
    pub staged: bool,
    /// Tiene algo sin preparar.
    pub unstaged: bool,
}

/// Rama, upstream y distancia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BranchState {
    /// Commit de HEAD; falta si la rama no tiene commits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub head_sha: Option<String>,
    /// Nombre corto de la rama; falta con HEAD separado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub branch: Option<String>,
    /// Upstream (`origin/main`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub upstream: Option<String>,
    /// Commits por delante del upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ahead: Option<u32>,
    /// Commits por detras del upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub behind: Option<u32>,
    /// HEAD separado.
    pub is_detached: bool,
    /// La rama no tiene commits todavia.
    pub is_unborn: bool,
}

/// Ficheros de cada grupo del panel de staging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StatusCounts {
    /// Con algo preparado.
    pub staged: u32,
    /// Con algo sin preparar.
    pub unstaged: u32,
    /// Sin seguimiento.
    pub untracked: u32,
    /// En conflicto.
    pub conflicted: u32,
}

/// Lo que esta a medias.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum Operation {
    /// Un merge.
    Merge {
        /// Commit que entra.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        incoming: Option<String>,
        /// Rama que entra.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        incoming_name: Option<String>,
    },
    /// Un rebase.
    Rebase {
        /// Rama que se rebasa.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        branch: Option<String>,
        /// Commit sobre el que se rebasa.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        onto: Option<String>,
        /// Paso actual, empezando en 1.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        step: Option<u32>,
        /// Pasos en total.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        total: Option<u32>,
        /// Commit en el que se paro.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        stopped_at: Option<String>,
        /// Rebase interactivo.
        interactive: bool,
    },
    /// Un cherry-pick.
    CherryPick {
        /// Commit que se aplica.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        commit: Option<String>,
    },
    /// Un revert.
    Revert {
        /// Commit que se deshace.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        commit: Option<String>,
    },
}

/// Resultado de `status/read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkingTreeStatus {
    /// Repositorio.
    pub repo_id: String,
    /// Rama y upstream.
    pub branch: BranchState,
    /// Ficheros con cambios, en el orden de git.
    pub entries: Vec<StatusEntry>,
    /// Cuantos hay en cada grupo.
    pub counts: StatusCounts,
    /// Operacion a medias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub operation: Option<Operation>,
}
