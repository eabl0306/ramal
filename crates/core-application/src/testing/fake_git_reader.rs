//! `FakeGitReader`: responde lo que se le diga y cuenta las llamadas.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use ramal_core_domain::{CommitRecord, Head, RefName, RefSnapshot, RepoRoot};

use super::samples::version_info;
use crate::port::git_reader::{GitReadError, GitReader, GitVersion, GitVersionInfo, LogQuery};

/// Cuantas veces se llamo a cada metodo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FakeCalls {
    /// Llamadas a `version`.
    pub version: usize,
    /// Llamadas a `discover`.
    pub discover: usize,
    /// Llamadas a `read_refs`.
    pub read_refs: usize,
    /// Llamadas a `read_log`.
    pub read_log: usize,
}

#[derive(Debug)]
struct State {
    version: Result<GitVersionInfo, GitReadError>,
    repositories: HashMap<PathBuf, RepoRoot>,
    discover_error: Option<GitReadError>,
    refs: Result<RefSnapshot, GitReadError>,
    log: Result<Vec<CommitRecord>, GitReadError>,
    calls: FakeCalls,
    last_log_query: Option<LogQuery>,
}

/// Lector de git falso. Por defecto: git 2.45, ningun repositorio, refs vacias
/// con HEAD en una rama `main` sin nacer y log vacio.
#[derive(Debug)]
pub struct FakeGitReader {
    state: Mutex<State>,
}

impl Default for FakeGitReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeGitReader {
    /// Lector con los valores por defecto.
    #[must_use]
    pub fn new() -> Self {
        let head = Head::Branch {
            name: RefName::parse("refs/heads/main").unwrap_or_else(|_| unreachable!()),
            target: None,
        };
        Self {
            state: Mutex::new(State {
                version: Ok(version_info(GitVersion::new(2, 45, 0))),
                repositories: HashMap::new(),
                discover_error: None,
                refs: Ok(RefSnapshot::new(Vec::new(), head, None)),
                log: Ok(Vec::new()),
                calls: FakeCalls::default(),
                last_log_query: None,
            }),
        }
    }

    /// Fija la respuesta de `version`.
    #[must_use]
    pub fn with_version(self, version: Result<GitVersionInfo, GitReadError>) -> Self {
        self.lock().version = version;
        self
    }

    /// Hace que `discover(path)` devuelva `root`.
    #[must_use]
    pub fn with_repository(self, path: impl Into<PathBuf>, root: RepoRoot) -> Self {
        self.lock().repositories.insert(path.into(), root);
        self
    }

    /// Hace que `discover` falle con `error` para cualquier ruta.
    #[must_use]
    pub fn with_discover_error(self, error: GitReadError) -> Self {
        self.lock().discover_error = Some(error);
        self
    }

    /// Fija la respuesta de `read_refs`.
    #[must_use]
    pub fn with_refs(self, refs: Result<RefSnapshot, GitReadError>) -> Self {
        self.lock().refs = refs;
        self
    }

    /// Cambia las refs de un lector ya compartido, como si alguien hiciera un
    /// commit o un fetch entre dos llamadas.
    pub fn set_refs(&self, refs: RefSnapshot) {
        self.lock().refs = Ok(refs);
    }

    /// Fija la respuesta de `read_log`. Se recorta a `max_count` como haria git.
    #[must_use]
    pub fn with_log(self, log: Result<Vec<CommitRecord>, GitReadError>) -> Self {
        self.lock().log = log;
        self
    }

    /// Llamadas recibidas hasta ahora.
    #[must_use]
    pub fn calls(&self) -> FakeCalls {
        self.lock().calls
    }

    /// Ultima consulta de log recibida.
    #[must_use]
    pub fn last_log_query(&self) -> Option<LogQuery> {
        self.lock().last_log_query
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl GitReader for FakeGitReader {
    async fn version(&self) -> Result<GitVersionInfo, GitReadError> {
        let mut state = self.lock();
        state.calls.version += 1;
        state.version.clone()
    }

    async fn discover(&self, path: &Path) -> Result<RepoRoot, GitReadError> {
        let mut state = self.lock();
        state.calls.discover += 1;
        if let Some(error) = &state.discover_error {
            return Err(error.clone());
        }
        state
            .repositories
            .get(path)
            .cloned()
            .ok_or_else(|| GitReadError::NotARepository {
                path: path.to_path_buf(),
            })
    }

    async fn read_refs(&self, _repo: &RepoRoot) -> Result<RefSnapshot, GitReadError> {
        let mut state = self.lock();
        state.calls.read_refs += 1;
        state.refs.clone()
    }

    async fn read_log(
        &self,
        _repo: &RepoRoot,
        query: &LogQuery,
    ) -> Result<Vec<CommitRecord>, GitReadError> {
        let mut state = self.lock();
        state.calls.read_log += 1;
        state.last_log_query = Some(*query);
        state
            .log
            .clone()
            .map(|log| log.into_iter().take(query.max_count).collect())
    }
}
