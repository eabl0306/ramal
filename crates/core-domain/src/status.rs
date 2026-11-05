//! Estado del arbol de trabajo, tal como lo da `git status --porcelain=v2`.
//!
//! Es la base de la fila WIP del grafo, del panel de staging y de la barra de
//! estado. Cada entrada dice que cambio en el indice (lo preparado) y que en el
//! arbol de trabajo (lo que no), y los conflictos llevan su par de letras de git
//! (`UU`, `AA`, `DU`...) para que la interfaz sepa que resolver.

use crate::operation::Operation;
use crate::sha::Sha;

/// Que le paso a un fichero en un lado (indice o arbol de trabajo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    /// Sin cambios en este lado.
    Unmodified,
    /// Contenido cambiado.
    Modified,
    /// Cambio de tipo (fichero a enlace, por ejemplo).
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

/// Las siete formas de un fichero en conflicto, con sus letras de git.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConflictCode {
    /// `DD`: borrado en los dos lados (suele venir de un renombrado en ambos).
    BothDeleted,
    /// `AU`: anadido por nosotros (el otro lado lo renombro o lo borro).
    AddedByUs,
    /// `UD`: lo borraron ellos y nosotros lo cambiamos.
    DeletedByThem,
    /// `UA`: anadido por ellos.
    AddedByThem,
    /// `DU`: lo borramos nosotros y ellos lo cambiaron.
    DeletedByUs,
    /// `AA`: anadido en los dos lados con contenido distinto.
    BothAdded,
    /// `UU`: cambiado en los dos lados.
    BothModified,
}

impl ConflictCode {
    /// Un lado borro el fichero: se resuelve eligiendo, no fusionando lineas.
    #[must_use]
    pub const fn is_deletion(self) -> bool {
        matches!(
            self,
            Self::BothDeleted | Self::DeletedByThem | Self::DeletedByUs
        )
    }

    /// Solo un lado tiene el fichero en su ruta: es lo que deja un renombrado
    /// en un lado y un cambio o un borrado en el otro.
    #[must_use]
    pub const fn is_rename_side(self) -> bool {
        matches!(self, Self::AddedByUs | Self::AddedByThem)
    }
}

/// Lo que cambio dentro de un submodulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SubmoduleChange {
    /// Apunta a otro commit que el registrado.
    pub commit_changed: bool,
    /// Tiene ficheros rastreados con cambios.
    pub has_tracked_changes: bool,
    /// Tiene ficheros sin seguimiento.
    pub has_untracked: bool,
}

/// En que estado esta un fichero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryState {
    /// Rastreado y con cambios en el indice, en el arbol o en los dos.
    Changed {
        /// Cambio preparado.
        index: ChangeKind,
        /// Cambio sin preparar.
        worktree: ChangeKind,
        /// Si es un submodulo, que cambio dentro.
        submodule: Option<SubmoduleChange>,
    },
    /// En conflicto.
    Unmerged {
        /// Que forma tiene el conflicto.
        conflict: ConflictCode,
    },
    /// Sin seguimiento.
    Untracked,
    /// Ignorado (solo si se piden).
    Ignored,
}

/// Un fichero con algo que contar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// Ruta relativa a la raiz, con `/`.
    pub path: String,
    /// Ruta de origen de un renombrado o una copia.
    pub original_path: Option<String>,
    /// Estado.
    pub state: EntryState,
}

impl StatusEntry {
    /// Tiene algo preparado en el indice.
    #[must_use]
    pub const fn is_staged(&self) -> bool {
        matches!(self.state, EntryState::Changed { index, .. } if !matches!(index, ChangeKind::Unmodified))
    }

    /// Tiene algo sin preparar en el arbol de trabajo (un submodulo cuenta por
    /// sus cambios de dentro aunque el commit no cambie).
    #[must_use]
    pub fn is_unstaged(&self) -> bool {
        match self.state {
            EntryState::Changed {
                worktree,
                submodule,
                ..
            } => {
                worktree != ChangeKind::Unmodified
                    || submodule
                        .is_some_and(|change| change.has_tracked_changes || change.has_untracked)
            }
            _ => false,
        }
    }
}

/// Rama, upstream y distancia, de las cabeceras `# branch.*`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BranchStatus {
    /// Commit de HEAD; `None` si la rama todavia no tiene commits.
    pub head_sha: Option<Sha>,
    /// Nombre corto de la rama; `None` con HEAD separado.
    pub branch: Option<String>,
    /// Upstream (`origin/main`), si la rama tiene.
    pub upstream: Option<String>,
    /// Commits por delante y por detras del upstream, si se pudo calcular.
    pub ahead_behind: Option<AheadBehind>,
}

impl BranchStatus {
    /// HEAD apunta a un commit y no a una rama.
    #[must_use]
    pub const fn is_detached(&self) -> bool {
        self.branch.is_none()
    }

    /// La rama no tiene commits todavia.
    #[must_use]
    pub const fn is_unborn(&self) -> bool {
        self.head_sha.is_none()
    }
}

/// Distancia entre una rama y otra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AheadBehind {
    /// Commits que tiene la rama y no la otra.
    pub ahead: u32,
    /// Commits que tiene la otra y no la rama.
    pub behind: u32,
}

/// Cuantos ficheros hay en cada grupo del panel de staging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusCounts {
    /// Con algo preparado.
    pub staged: usize,
    /// Con algo sin preparar.
    pub unstaged: usize,
    /// Sin seguimiento.
    pub untracked: usize,
    /// En conflicto.
    pub conflicted: usize,
}

/// Estado completo del arbol de trabajo.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkingTreeStatus {
    /// Rama y upstream.
    pub branch: BranchStatus,
    /// Ficheros con cambios, en el orden de git.
    pub entries: Vec<StatusEntry>,
    /// Merge, rebase, cherry-pick o revert a medias.
    pub operation: Option<Operation>,
}

impl WorkingTreeStatus {
    /// Cuenta los ficheros de cada grupo. Un fichero con cambios preparados y sin
    /// preparar cuenta en los dos.
    #[must_use]
    pub fn counts(&self) -> StatusCounts {
        self.entries
            .iter()
            .fold(StatusCounts::default(), |mut counts, entry| {
                match entry.state {
                    EntryState::Unmerged { .. } => counts.conflicted += 1,
                    EntryState::Untracked => counts.untracked += 1,
                    EntryState::Ignored => {}
                    EntryState::Changed { .. } => {
                        counts.staged += usize::from(entry.is_staged());
                        counts.unstaged += usize::from(entry.is_unstaged());
                    }
                }
                counts
            })
    }

    /// No hay nada que preparar, ni conflictos, ni ficheros nuevos.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.counts() == StatusCounts::default()
    }
}

#[cfg(test)]
mod tests;
