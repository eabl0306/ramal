//! Refs, stashes, submodulos y conflictos.

use std::sync::Arc;

use git_rudder_core_domain::conflict::ConflictFile;
use git_rudder_core_domain::refs::RefListing;
use git_rudder_core_domain::stash::{StashContents, StashEntry};
use git_rudder_core_domain::submodule::Submodule;
use git_rudder_core_domain::{RepoId, RepoPath, Sha};

use super::debug_use_case;
use crate::error::EngineError;
use crate::port::repo_reader::{ConflictReader, InventoryReader};
use crate::state::open_repositories::OpenRepositories;

/// Ramas con upstream, tags y remotos.
pub struct ListRefs {
    inventory: Arc<dyn InventoryReader>,
    registry: Arc<OpenRepositories>,
}

impl ListRefs {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(inventory: Arc<dyn InventoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self {
            inventory,
            registry,
        }
    }

    /// Todo lo que lista la barra lateral, cada grupo por nombre.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, repo_id: &RepoId) -> Result<RefListing, EngineError> {
        let repo = self.registry.require(repo_id)?;
        Ok(self.inventory.list_refs(&repo.root).await?.sorted())
    }
}

/// Lista de stashes.
pub struct ListStashes {
    inventory: Arc<dyn InventoryReader>,
    registry: Arc<OpenRepositories>,
}

impl ListStashes {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(inventory: Arc<dyn InventoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self {
            inventory,
            registry,
        }
    }

    /// Stashes, el mas reciente primero.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, repo_id: &RepoId) -> Result<Vec<StashEntry>, EngineError> {
        let repo = self.registry.require(repo_id)?;
        Ok(self.inventory.list_stashes(&repo.root).await?)
    }
}

/// Contenido de un stash antes de aplicarlo.
pub struct ShowStash {
    inventory: Arc<dyn InventoryReader>,
    registry: Arc<OpenRepositories>,
}

impl ShowStash {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(inventory: Arc<dyn InventoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self {
            inventory,
            registry,
        }
    }

    /// Ficheros y diff del stash `sha`. Solo se leen shas que estan en la lista:
    /// leer un commit cualquiera como si fuera un stash daria un diff sin sentido.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::NotFound`] si `sha` no es un stash y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, repo_id: &RepoId, sha: Sha) -> Result<StashContents, EngineError> {
        let repo = self.registry.require(repo_id)?;
        let stashes = self.inventory.list_stashes(&repo.root).await?;
        if !stashes.iter().any(|stash| stash.sha == sha) {
            return Err(EngineError::NotFound {
                what: format!("stash {sha}"),
            });
        }
        Ok(self
            .inventory
            .stash_contents(&repo.root, repo.trust, sha)
            .await?)
    }
}

/// Submodulos, los anidados incluidos.
pub struct ListSubmodules {
    inventory: Arc<dyn InventoryReader>,
    registry: Arc<OpenRepositories>,
}

impl ListSubmodules {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(inventory: Arc<dyn InventoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self {
            inventory,
            registry,
        }
    }

    /// Submodulos por ruta.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, repo_id: &RepoId) -> Result<Vec<Submodule>, EngineError> {
        let repo = self.registry.require(repo_id)?;
        let mut submodules = self.inventory.list_submodules(&repo.root).await?;
        submodules.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(submodules)
    }
}

/// Lados y bloques de un fichero en conflicto.
pub struct ReadConflict {
    conflicts: Arc<dyn ConflictReader>,
    registry: Arc<OpenRepositories>,
}

impl ReadConflict {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(conflicts: Arc<dyn ConflictReader>, registry: Arc<OpenRepositories>) -> Self {
        Self {
            conflicts,
            registry,
        }
    }

    /// Conflicto de `path`.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::NotFound`] si el fichero no esta en conflicto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        path: &RepoPath,
    ) -> Result<ConflictFile, EngineError> {
        let repo = self.registry.require(repo_id)?;
        Ok(self.conflicts.read_conflict(&repo.root, path).await?)
    }
}

debug_use_case!(
    ListRefs,
    ListStashes,
    ShowStash,
    ListSubmodules,
    ReadConflict
);
