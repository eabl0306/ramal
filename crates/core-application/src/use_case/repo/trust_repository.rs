//! `TrustRepository`: el usuario confia en un repositorio abierto.
//!
//! Solo lo llaman los hosts, despues de ensenar su propia confirmacion: si un
//! webview comprometido pudiera confiar, la pregunta no protegeria de nada. La
//! confianza se guarda en el dispositivo y no se sincroniza (ADR 0020).

use std::sync::Arc;

use git_rudder_core_domain::{RepoId, TrustLevel};

use crate::error::EngineError;
use crate::port::host::TrustStore;
use crate::state::open_repositories::OpenRepositories;

/// Confia en un repositorio abierto.
pub struct TrustRepository {
    store: Arc<dyn TrustStore>,
    registry: Arc<OpenRepositories>,
}

impl std::fmt::Debug for TrustRepository {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrustRepository")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl TrustRepository {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(store: Arc<dyn TrustStore>, registry: Arc<OpenRepositories>) -> Self {
        Self { store, registry }
    }

    /// Recuerda la confianza y la aplica ya a las lecturas siguientes.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::Internal`] si no se puede guardar.
    pub async fn execute(&self, repo_id: &RepoId) -> Result<(), EngineError> {
        let repo = self.registry.require(repo_id)?;
        self.store
            .trust(&repo.root)
            .await
            .map_err(|error| EngineError::Internal {
                detail: error.to_string(),
            })?;
        self.registry.set_trust(repo_id, TrustLevel::Trusted);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
