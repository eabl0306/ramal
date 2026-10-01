//! `OpenRepository`: descubre la raiz del repositorio que contiene una ruta, lo
//! registra y emite su `RepoId`.
//!
//! Aqui se aplica el git minimo: antes de tocar un repositorio se comprueba la
//! version, y un git anterior a 2.34 da `GitVersionUnsupported` en vez de fallos
//! raros mas adelante. Solo lo llaman los hosts: la ruta nunca viene de un webview.

use std::path::Path;
use std::sync::Arc;

use ramal_core_domain::{RefSnapshot, RepoId, RepoRoot};

use crate::error::EngineError;
use crate::port::git_reader::{GitReader, MINIMUM_GIT_VERSION};
use crate::state::open_repositories::OpenRepositories;

/// Repositorio recien abierto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedRepository {
    /// Id con el que la interfaz lo pide a partir de ahora.
    pub id: RepoId,
    /// Raiz del arbol de trabajo.
    pub root: RepoRoot,
    /// Refs y HEAD en el momento de abrirlo.
    pub refs: RefSnapshot,
}

/// Abre un repositorio.
pub struct OpenRepository {
    git: Arc<dyn GitReader>,
    registry: Arc<OpenRepositories>,
}

impl std::fmt::Debug for OpenRepository {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenRepository")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl OpenRepository {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(git: Arc<dyn GitReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { git, registry }
    }

    /// Abre el repositorio que contiene `path`.
    ///
    /// # Errors
    ///
    /// [`EngineError::GitUnavailable`] sin git, [`EngineError::GitVersionUnsupported`]
    /// con un git anterior al minimo, [`EngineError::NotARepository`] si la ruta no
    /// esta en un repositorio y [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, path: &Path) -> Result<OpenedRepository, EngineError> {
        let git = self.git.version().await?;
        if !git.is_supported() {
            return Err(EngineError::GitVersionUnsupported {
                found: git.version,
                minimum: MINIMUM_GIT_VERSION,
            });
        }
        let root = self.git.discover(path).await?;
        let refs = self.git.read_refs(&root).await?;
        let id = self.registry.register(root.clone());
        Ok(OpenedRepository { id, root, refs })
    }
}

#[cfg(test)]
mod tests;
