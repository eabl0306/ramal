//! Estado del arbol de trabajo y diff de un fichero.

use std::sync::Arc;

use git_rudder_core_domain::RepoId;
use git_rudder_core_domain::diff::{DiffOptions, FileDiff};
use git_rudder_core_domain::status::WorkingTreeStatus;

use super::debug_use_case;
use crate::error::EngineError;
use crate::port::repo_reader::{DiffRequest, StatusReader};
use crate::state::open_repositories::OpenRepositories;

/// Estado del arbol de trabajo, con la rama y la operacion a medias.
pub struct ReadStatus {
    status: Arc<dyn StatusReader>,
    registry: Arc<OpenRepositories>,
}

impl ReadStatus {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(status: Arc<dyn StatusReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { status, registry }
    }

    /// Estado de `repo_id`. Es una lectura de fondo: nunca toma `index.lock`.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, repo_id: &RepoId) -> Result<WorkingTreeStatus, EngineError> {
        let repo = self.registry.require(repo_id)?;
        Ok(self.status.read_status(&repo.root, repo.trust).await?)
    }
}

/// Diff de un fichero.
pub struct ReadFileDiff {
    status: Arc<dyn StatusReader>,
    registry: Arc<OpenRepositories>,
}

impl ReadFileDiff {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(status: Arc<dyn StatusReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { status, registry }
    }

    /// Diff pedido. El contexto se recorta a [`DiffOptions::MAX_CONTEXT_LINES`].
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::NotFound`] si el fichero o el commit no existen y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        mut request: DiffRequest,
    ) -> Result<FileDiff, EngineError> {
        let repo = self.registry.require(repo_id)?;
        request.options.context_lines = request
            .options
            .context_lines
            .min(DiffOptions::MAX_CONTEXT_LINES);
        Ok(self
            .status
            .read_diff(&repo.root, repo.trust, &request)
            .await?)
    }
}

debug_use_case!(ReadStatus, ReadFileDiff);
