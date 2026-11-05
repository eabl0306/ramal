//! Puertos de lectura de un repositorio abierto, partidos por area: estado y
//! diffs, historia, inventario (refs, stashes, submodulos), conflictos y la
//! inspeccion previa a confiar.
//!
//! Son puertos distintos del [`GitReader`](super::git_reader::GitReader) del
//! primer corte para que cada fake se lea de un vistazo y para que una lectura
//! caliente pueda pasar algun dia a `gix` sin tocar las demas. El adaptador del
//! CLI los implementa todos.
//!
//! Los que pueden lanzar la configuracion del repositorio (`git status` lanza
//! `core.fsmonitor`, un diff los `textconv`) reciben el [`TrustLevel`]: sin
//! confianza, el adaptador los apaga en la linea de comandos (ADR 0020).

use async_trait::async_trait;
use git_rudder_core_domain::blame::Blame;
use git_rudder_core_domain::conflict::ConflictFile;
use git_rudder_core_domain::diff::{DiffOptions, DiffTarget, FileDiff};
use git_rudder_core_domain::history::{BranchComparison, FileHistoryEntry, MatchKind, SearchQuery};
use git_rudder_core_domain::refs::RefListing;
use git_rudder_core_domain::stash::{StashContents, StashEntry};
use git_rudder_core_domain::status::WorkingTreeStatus;
use git_rudder_core_domain::submodule::Submodule;
use git_rudder_core_domain::trust::TrustInspection;
use git_rudder_core_domain::{CommitDetail, CommitRecord, RepoPath, RepoRoot, Sha, TrustLevel};

use super::git_reader::GitReadError;

/// Un diff de un fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffRequest {
    /// Que lados comparar.
    pub target: DiffTarget,
    /// Fichero.
    pub path: RepoPath,
    /// Ruta en el lado viejo si se renombro.
    pub old_path: Option<RepoPath>,
    /// Espacios, palabras y contexto.
    pub options: DiffOptions,
}

/// Estado del arbol de trabajo y diffs.
#[async_trait]
pub trait StatusReader: Send + Sync {
    /// Estado con la rama, los ficheros y la operacion a medias.
    async fn read_status(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
    ) -> Result<WorkingTreeStatus, GitReadError>;

    /// Diff de un fichero.
    async fn read_diff(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        request: &DiffRequest,
    ) -> Result<FileDiff, GitReadError>;
}

/// Un commit encontrado por la busqueda, sin su fila (la pone el caso de uso).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundCommit {
    /// El commit.
    pub commit: CommitRecord,
    /// Por que coincide.
    pub matched: MatchKind,
}

/// Historia: detalle, busqueda, ficheros, blame y comparaciones.
#[async_trait]
pub trait HistoryReader: Send + Sync {
    /// Sha del commit al que lleva una revision (sha, rama, tag).
    async fn resolve_commit(&self, repo: &RepoRoot, revision: &str) -> Result<Sha, GitReadError>;

    /// Detalle de un commit.
    async fn commit_detail(&self, repo: &RepoRoot, sha: Sha) -> Result<CommitDetail, GitReadError>;

    /// Commits de todo el historial (las mismas refs que el grafo) que
    /// coinciden, en orden topologico y como mucho `limit`.
    async fn search(
        &self,
        repo: &RepoRoot,
        query: &SearchQuery,
        limit: usize,
    ) -> Result<Vec<FoundCommit>, GitReadError>;

    /// Commits que tocan un fichero siguiendo renombrados, saltando `skip` y
    /// como mucho `limit`.
    async fn file_history(
        &self,
        repo: &RepoRoot,
        path: &RepoPath,
        skip: usize,
        limit: usize,
    ) -> Result<Vec<FileHistoryEntry>, GitReadError>;

    /// Blame de un fichero en un commit, o en el arbol de trabajo si no hay.
    async fn blame(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        path: &RepoPath,
        revision: Option<Sha>,
    ) -> Result<Blame, GitReadError>;

    /// Compara dos commits desde su ancestro comun.
    async fn compare(
        &self,
        repo: &RepoRoot,
        base: Sha,
        head: Sha,
    ) -> Result<BranchComparison, GitReadError>;
}

/// Inventario del repositorio: refs, stashes y submodulos.
#[async_trait]
pub trait InventoryReader: Send + Sync {
    /// Ramas con su upstream, tags y remotos.
    async fn list_refs(&self, repo: &RepoRoot) -> Result<RefListing, GitReadError>;

    /// Stashes, el mas reciente primero.
    async fn list_stashes(&self, repo: &RepoRoot) -> Result<Vec<StashEntry>, GitReadError>;

    /// Ficheros y diff de un stash.
    async fn stash_contents(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        stash: Sha,
    ) -> Result<StashContents, GitReadError>;

    /// Submodulos, los anidados incluidos.
    async fn list_submodules(&self, repo: &RepoRoot) -> Result<Vec<Submodule>, GitReadError>;
}

/// Ficheros en conflicto.
#[async_trait]
pub trait ConflictReader: Send + Sync {
    /// Lados y bloques de un fichero en conflicto.
    async fn read_conflict(
        &self,
        repo: &RepoRoot,
        path: &RepoPath,
    ) -> Result<ConflictFile, GitReadError>;
}

/// Mira lo que podria ejecutar codigo sin ejecutar nada.
#[async_trait]
pub trait RepoInspector: Send + Sync {
    /// Configuracion local (sin seguir `include`) y ficheros de la carpeta de
    /// hooks.
    async fn inspect(&self, repo: &RepoRoot) -> Result<TrustInspection, GitReadError>;
}
