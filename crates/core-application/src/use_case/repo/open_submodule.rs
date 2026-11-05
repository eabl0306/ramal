//! `OpenSubmodule`: abre como repositorio un submodulo de uno abierto.
//!
//! La ruta es relativa y viene de la interfaz, asi que solo se abre si es la de
//! un submodulo inicializado de la lista: un webview no puede usar este metodo
//! para abrir cualquier carpeta del disco. El submodulo pasa por la misma
//! decision de confianza que cualquier repositorio.

use std::sync::Arc;

use git_rudder_core_domain::{RepoId, RepoPath};

use super::open_repository::{OpenRepository, OpenedRepository};
use crate::error::EngineError;
use crate::port::repo_reader::InventoryReader;
use crate::state::open_repositories::OpenRepositories;

/// Abre un submodulo.
pub struct OpenSubmodule {
    inventory: Arc<dyn InventoryReader>,
    registry: Arc<OpenRepositories>,
    open: Arc<OpenRepository>,
}

impl std::fmt::Debug for OpenSubmodule {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenSubmodule")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl OpenSubmodule {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(
        inventory: Arc<dyn InventoryReader>,
        registry: Arc<OpenRepositories>,
        open: Arc<OpenRepository>,
    ) -> Self {
        Self {
            inventory,
            registry,
            open,
        }
    }

    /// Abre el submodulo `path` de `repo_id`.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el padre no esta abierto,
    /// [`EngineError::NotFound`] si `path` no es un submodulo inicializado, y los
    /// de [`OpenRepository::execute`].
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        path: &RepoPath,
    ) -> Result<OpenedRepository, EngineError> {
        let parent = self.registry.require(repo_id)?;
        let submodules = self.inventory.list_submodules(&parent.root).await?;
        let known = submodules
            .iter()
            .any(|submodule| submodule.path == path.as_str() && submodule.can_open());
        if !known {
            return Err(EngineError::NotFound {
                what: format!("submodule {path}"),
            });
        }
        let dir = parent.root.as_path().join(path.as_str());
        self.open.execute(&dir, false).await
    }
}

#[cfg(test)]
mod tests;
