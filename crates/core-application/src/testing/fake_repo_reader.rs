//! `FakeRepoReader`: un fake para todos los puertos de lectura de
//! [`repo_reader`](crate::port::repo_reader).
//!
//! Cada respuesta se guarda por su tipo (`with(Ok(estado))`), porque cada metodo
//! devuelve un tipo distinto, y cada llamada se cuenta por metodo con sus
//! argumentos en texto para poder comprobarlos.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use git_rudder_core_domain::blame::Blame;
use git_rudder_core_domain::conflict::ConflictFile;
use git_rudder_core_domain::diff::FileDiff;
use git_rudder_core_domain::history::{BranchComparison, FileHistoryEntry, SearchQuery};
use git_rudder_core_domain::refs::RefListing;
use git_rudder_core_domain::stash::{StashContents, StashEntry};
use git_rudder_core_domain::status::WorkingTreeStatus;
use git_rudder_core_domain::submodule::Submodule;
use git_rudder_core_domain::trust::TrustInspection;
use git_rudder_core_domain::{CommitDetail, RepoPath, RepoRoot, Sha, TrustLevel};

use crate::port::git_reader::GitReadError;
use crate::port::repo_reader::{
    ConflictReader, DiffRequest, FoundCommit, HistoryReader, InventoryReader, RepoInspector,
    StatusReader,
};

#[derive(Default)]
struct State {
    answers: HashMap<TypeId, Box<dyn Any + Send>>,
    calls: HashMap<&'static str, usize>,
    last_args: HashMap<&'static str, String>,
}

/// Lector falso: responde lo que se le diga por tipo y cuenta las llamadas. Sin
/// respuesta para un tipo, falla con [`GitReadError::Failed`].
#[derive(Default)]
pub struct FakeRepoReader {
    state: Mutex<State>,
}

impl std::fmt::Debug for FakeRepoReader {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FakeRepoReader")
            .field("calls", &self.lock().calls)
            .finish_non_exhaustive()
    }
}

impl FakeRepoReader {
    /// Lector sin respuestas.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Respuesta de los metodos que devuelven `T`.
    #[must_use]
    pub fn with<T: Clone + Send + 'static>(self, answer: Result<T, GitReadError>) -> Self {
        self.set(answer);
        self
    }

    /// Cambia la respuesta de un lector ya compartido.
    pub fn set<T: Clone + Send + 'static>(&self, answer: Result<T, GitReadError>) {
        self.lock()
            .answers
            .insert(TypeId::of::<T>(), Box::new(answer));
    }

    /// Veces que se llamo a `method`.
    #[must_use]
    pub fn calls(&self, method: &str) -> usize {
        self.lock().calls.get(method).copied().unwrap_or(0)
    }

    /// Argumentos de la ultima llamada a `method`, en texto.
    #[must_use]
    pub fn last_args(&self, method: &str) -> Option<String> {
        self.lock().last_args.get(method).cloned()
    }

    fn reply<T: Clone + 'static>(
        &self,
        method: &'static str,
        args: String,
    ) -> Result<T, GitReadError> {
        let mut state = self.lock();
        *state.calls.entry(method).or_default() += 1;
        state.last_args.insert(method, args);
        state
            .answers
            .get(&TypeId::of::<T>())
            .and_then(|answer| answer.downcast_ref::<Result<T, GitReadError>>())
            .cloned()
            .unwrap_or_else(|| {
                Err(GitReadError::Failed {
                    detail: format!("the fake has no answer for {method}"),
                })
            })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl StatusReader for FakeRepoReader {
    async fn read_status(
        &self,
        _repo: &RepoRoot,
        trust: TrustLevel,
    ) -> Result<WorkingTreeStatus, GitReadError> {
        self.reply("read_status", format!("{trust:?}"))
    }

    async fn read_diff(
        &self,
        _repo: &RepoRoot,
        trust: TrustLevel,
        request: &DiffRequest,
    ) -> Result<FileDiff, GitReadError> {
        self.reply("read_diff", format!("{trust:?} {request:?}"))
    }
}

#[async_trait]
impl HistoryReader for FakeRepoReader {
    async fn resolve_commit(&self, _repo: &RepoRoot, revision: &str) -> Result<Sha, GitReadError> {
        self.reply("resolve_commit", revision.to_owned())
    }

    async fn commit_detail(
        &self,
        _repo: &RepoRoot,
        sha: Sha,
    ) -> Result<CommitDetail, GitReadError> {
        self.reply("commit_detail", sha.to_hex())
    }

    async fn search(
        &self,
        _repo: &RepoRoot,
        query: &SearchQuery,
        limit: usize,
    ) -> Result<Vec<FoundCommit>, GitReadError> {
        self.reply("search", format!("{} {limit}", query.text()))
    }

    async fn file_history(
        &self,
        _repo: &RepoRoot,
        path: &RepoPath,
        skip: usize,
        limit: usize,
    ) -> Result<Vec<FileHistoryEntry>, GitReadError> {
        self.reply("file_history", format!("{path} {skip} {limit}"))
    }

    async fn blame(
        &self,
        _repo: &RepoRoot,
        trust: TrustLevel,
        path: &RepoPath,
        revision: Option<Sha>,
    ) -> Result<Blame, GitReadError> {
        self.reply("blame", format!("{trust:?} {path} {revision:?}"))
    }

    async fn compare(
        &self,
        _repo: &RepoRoot,
        base: Sha,
        head: Sha,
    ) -> Result<BranchComparison, GitReadError> {
        self.reply("compare", format!("{base} {head}"))
    }
}

#[async_trait]
impl InventoryReader for FakeRepoReader {
    async fn list_refs(&self, _repo: &RepoRoot) -> Result<RefListing, GitReadError> {
        self.reply("list_refs", String::new())
    }

    async fn list_stashes(&self, _repo: &RepoRoot) -> Result<Vec<StashEntry>, GitReadError> {
        self.reply("list_stashes", String::new())
    }

    async fn stash_contents(
        &self,
        _repo: &RepoRoot,
        trust: TrustLevel,
        stash: Sha,
    ) -> Result<StashContents, GitReadError> {
        self.reply("stash_contents", format!("{trust:?} {stash}"))
    }

    async fn list_submodules(&self, _repo: &RepoRoot) -> Result<Vec<Submodule>, GitReadError> {
        self.reply("list_submodules", String::new())
    }
}

#[async_trait]
impl ConflictReader for FakeRepoReader {
    async fn read_conflict(
        &self,
        _repo: &RepoRoot,
        path: &RepoPath,
    ) -> Result<ConflictFile, GitReadError> {
        self.reply("read_conflict", path.to_string())
    }
}

#[async_trait]
impl RepoInspector for FakeRepoReader {
    async fn inspect(&self, repo: &RepoRoot) -> Result<TrustInspection, GitReadError> {
        self.reply("inspect", repo.as_path().display().to_string())
    }
}
