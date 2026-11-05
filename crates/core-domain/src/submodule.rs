//! Submodulos de un repositorio, anidados incluidos.
//!
//! La barra lateral los lista con su estado y deja abrir cada uno como un
//! repositorio mas.

use crate::sha::Sha;

/// Estado de un submodulo respecto a lo que registra el repositorio padre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubmoduleState {
    /// Registrado pero sin clonar (`git submodule update --init` pendiente).
    Uninitialized,
    /// En el commit que registra el padre.
    UpToDate,
    /// En otro commit.
    Modified,
    /// En conflicto en el padre.
    Conflicted,
}

/// Un submodulo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submodule {
    /// Ruta relativa a la raiz del repositorio abierto, con `/`.
    pub path: String,
    /// Commit en el que esta (o el registrado, si no esta inicializado).
    pub sha: Sha,
    /// Estado.
    pub state: SubmoduleState,
    /// Descripcion que da git (`v1.2-3-gabc1234`), si tiene.
    pub describe: Option<String>,
}

impl Submodule {
    /// Se puede abrir como repositorio: esta clonado.
    #[must_use]
    pub const fn can_open(&self) -> bool {
        !matches!(self.state, SubmoduleState::Uninitialized)
    }
}
