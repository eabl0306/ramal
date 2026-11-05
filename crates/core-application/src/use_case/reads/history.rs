//! Detalle de commit, historia de un fichero, blame y comparar ramas.

use std::sync::Arc;

use git_rudder_core_domain::blame::Blame;
use git_rudder_core_domain::graph::WindowRange;
use git_rudder_core_domain::history::{BranchComparison, FileHistoryEntry};
use git_rudder_core_domain::{CommitDetail, RepoId, RepoPath, Sha};

use super::debug_use_case;
use crate::error::EngineError;
use crate::port::repo_reader::HistoryReader;
use crate::state::open_repositories::OpenRepositories;

/// Detalle de un commit: mensaje, autor, fechas, padres y ficheros.
pub struct ReadCommitDetail {
    history: Arc<dyn HistoryReader>,
    registry: Arc<OpenRepositories>,
}

impl ReadCommitDetail {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(history: Arc<dyn HistoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { history, registry }
    }

    /// Detalle de `sha`.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::NotFound`] si el commit no existe y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(&self, repo_id: &RepoId, sha: Sha) -> Result<CommitDetail, EngineError> {
        let repo = self.registry.require(repo_id)?;
        Ok(self.history.commit_detail(&repo.root, sha).await?)
    }
}

/// Una pagina de la historia de un fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHistoryPage {
    /// Entradas, la mas reciente primero.
    pub entries: Vec<FileHistoryEntry>,
    /// Hay mas despues de esta pagina.
    pub has_more: bool,
}

/// Commits que tocan un fichero, siguiendo renombrados.
pub struct FileHistory {
    history: Arc<dyn HistoryReader>,
    registry: Arc<OpenRepositories>,
}

impl FileHistory {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(history: Arc<dyn HistoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { history, registry }
    }

    /// Entradas `[offset, offset + limit)`; `limit` va de 1 a 1000.
    ///
    /// # Errors
    ///
    /// [`EngineError::InvalidWindow`] si la pagina no es valida,
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        path: &RepoPath,
        offset: usize,
        limit: usize,
    ) -> Result<FileHistoryPage, EngineError> {
        let range = WindowRange::new(offset, limit)?;
        let repo = self.registry.require(repo_id)?;
        let mut entries = self
            .history
            .file_history(&repo.root, path, range.offset(), range.limit() + 1)
            .await?;
        let has_more = entries.len() > range.limit();
        entries.truncate(range.limit());
        Ok(FileHistoryPage { entries, has_more })
    }
}

/// Autor, commit y fecha de cada linea.
pub struct BlameFile {
    history: Arc<dyn HistoryReader>,
    registry: Arc<OpenRepositories>,
}

impl BlameFile {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(history: Arc<dyn HistoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { history, registry }
    }

    /// Blame de `path` en `revision`, o en el arbol de trabajo si no hay.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::NotFound`] si el fichero no existe ahi y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        path: &RepoPath,
        revision: Option<Sha>,
    ) -> Result<Blame, EngineError> {
        let repo = self.registry.require(repo_id)?;
        Ok(self
            .history
            .blame(&repo.root, repo.trust, path, revision)
            .await?)
    }
}

/// Compara dos refs desde su ancestro comun.
pub struct CompareRefs {
    history: Arc<dyn HistoryReader>,
    registry: Arc<OpenRepositories>,
}

impl CompareRefs {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(history: Arc<dyn HistoryReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { history, registry }
    }

    /// Lo que tiene `head` y no `base`, y al reves. `base` y `head` son una rama,
    /// un tag o un sha.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::NotFound`] si alguna revision no existe y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        base: &str,
        head: &str,
    ) -> Result<BranchComparison, EngineError> {
        let repo = self.registry.require(repo_id)?;
        let base = self.history.resolve_commit(&repo.root, base).await?;
        let head = self.history.resolve_commit(&repo.root, head).await?;
        Ok(self.history.compare(&repo.root, base, head).await?)
    }
}

debug_use_case!(ReadCommitDetail, FileHistory, BlameFile, CompareRefs);
