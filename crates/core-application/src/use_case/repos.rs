//! Casos de uso sobre repositorios que no estan abiertos.
//!
//! El estado de fondo de las tarjetas del inicio y buscar repositorios en una
//! carpeta. Los dos reciben rutas absolutas, asi que solo los llaman los hosts.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use git_rudder_core_domain::operation::Operation;
use git_rudder_core_domain::status::{BranchStatus, StatusCounts};
use git_rudder_core_domain::{RepoRoot, TrustLevel};

use crate::error::EngineError;
use crate::port::git_reader::GitReader;
use crate::port::host::{RepoScanner, TrustStore};
use crate::port::repo_reader::StatusReader;
use crate::state::concurrency::ConcurrencyLimit;

/// Procesos de git a la vez para el estado de fondo.
pub const BACKGROUND_STATUS_CONCURRENCY: usize = 4;

/// Profundidad maxima al buscar repositorios.
pub const MAX_SCAN_DEPTH: usize = 8;

/// Lo que ensena la tarjeta de un repositorio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSummary {
    /// Raiz del repositorio.
    pub root: RepoRoot,
    /// Rama, upstream y distancia.
    pub branch: BranchStatus,
    /// Ficheros de cada grupo.
    pub counts: StatusCounts,
    /// Operacion a medias, si la hay.
    pub operation: Option<Operation>,
}

/// Estado de fondo de un repositorio de la lista, con un limite de procesos de
/// git a la vez compartido por todas las llamadas.
///
/// Cada repositorio es una llamada aparte: uno lento solo retrasa la suya y
/// ocupa un turno, no la respuesta de todos.
pub struct SummarizeRepository {
    git: Arc<dyn GitReader>,
    status: Arc<dyn StatusReader>,
    trust_store: Arc<dyn TrustStore>,
    limit: ConcurrencyLimit,
}

impl std::fmt::Debug for SummarizeRepository {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SummarizeRepository")
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}

impl SummarizeRepository {
    /// Construye el caso de uso con un limite de `concurrency` a la vez.
    #[must_use]
    pub fn new(
        git: Arc<dyn GitReader>,
        status: Arc<dyn StatusReader>,
        trust_store: Arc<dyn TrustStore>,
        concurrency: usize,
    ) -> Self {
        Self {
            git,
            status,
            trust_store,
            limit: ConcurrencyLimit::new(concurrency),
        }
    }

    /// Estado del repositorio que contiene `path`. Sin abrirlo: no se registra
    /// ni se vigila, y si el usuario no confio en el se lee sin confianza.
    ///
    /// # Errors
    ///
    /// [`EngineError::NotARepository`] si la ruta no es de un repositorio,
    /// [`EngineError::GitUnavailable`] sin git y [`EngineError::GitFailed`] si
    /// git falla.
    pub async fn execute(&self, path: &Path) -> Result<RepoSummary, EngineError> {
        let _turn = self.limit.acquire().await;
        let root = self.git.discover(path).await?;
        let trust = match self.trust_store.is_trusted(&root).await {
            Ok(true) => TrustLevel::Trusted,
            Ok(false) | Err(_) => TrustLevel::Untrusted,
        };
        let status = self.status.read_status(&root, trust).await?;
        Ok(RepoSummary {
            counts: status.counts(),
            root,
            branch: status.branch,
            operation: status.operation,
        })
    }

    /// Lecturas en curso ahora mismo.
    #[must_use]
    pub fn in_flight(&self) -> usize {
        self.limit.in_use()
    }
}

/// Busca repositorios dentro de una carpeta.
pub struct ScanRepositories {
    scanner: Arc<dyn RepoScanner>,
}

impl std::fmt::Debug for ScanRepositories {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScanRepositories")
            .finish_non_exhaustive()
    }
}

impl ScanRepositories {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(scanner: Arc<dyn RepoScanner>) -> Self {
        Self { scanner }
    }

    /// Raices bajo `dir`, como mucho a `max_depth` (hasta [`MAX_SCAN_DEPTH`]),
    /// ordenadas y sin repetir. Cancelar la peticion para la busqueda.
    ///
    /// # Errors
    ///
    /// [`EngineError::NotFound`] si la carpeta no existe y
    /// [`EngineError::GitFailed`] si no se puede leer.
    pub async fn execute(&self, dir: &Path, max_depth: usize) -> Result<Vec<PathBuf>, EngineError> {
        let mut found = self
            .scanner
            .scan(dir, max_depth.min(MAX_SCAN_DEPTH))
            .await?;
        found.sort();
        found.dedup();
        Ok(found)
    }
}

#[cfg(test)]
mod tests;
