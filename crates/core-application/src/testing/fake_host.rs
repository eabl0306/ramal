//! Fakes de los puertos del host: eventos, vigilancia, confianza, busqueda de
//! repositorios y cambio de git.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use async_trait::async_trait;
use git_rudder_core_domain::{RepoId, RepoRoot};

use super::samples::version_info;
use crate::port::git_reader::{GitReadError, GitVersion, GitVersionInfo};
use crate::port::host::{
    EngineEvent, EventSink, GitSetup, RepoScanner, RepoWatcher, TrustStore, TrustStoreError,
    WatchError,
};

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Guarda los eventos que recibe.
#[derive(Debug, Default)]
pub struct RecordingSink {
    events: Mutex<Vec<EngineEvent>>,
}

impl RecordingSink {
    /// Eventos recibidos hasta ahora.
    #[must_use]
    pub fn events(&self) -> Vec<EngineEvent> {
        lock(&self.events).clone()
    }
}

impl EventSink for RecordingSink {
    fn publish(&self, event: EngineEvent) {
        lock(&self.events).push(event);
    }
}

/// Apunta que repositorios se pidio vigilar; puede fallar a proposito.
#[derive(Debug, Default)]
pub struct FakeWatcher {
    watched: Mutex<Vec<RepoId>>,
    fail: bool,
}

impl FakeWatcher {
    /// Un vigilante que siempre falla.
    #[must_use]
    pub fn failing() -> Self {
        Self {
            watched: Mutex::default(),
            fail: true,
        }
    }

    /// Ids que se pidio vigilar, en orden.
    #[must_use]
    pub fn watched(&self) -> Vec<RepoId> {
        lock(&self.watched).clone()
    }
}

impl RepoWatcher for FakeWatcher {
    fn watch(&self, repo_id: &RepoId, _root: &RepoRoot) -> Result<(), WatchError> {
        lock(&self.watched).push(repo_id.clone());
        if self.fail {
            return Err(WatchError("the fake watcher always fails".to_owned()));
        }
        Ok(())
    }
}

/// Confianza en memoria; puede fallar a proposito.
#[derive(Debug, Default)]
pub struct MemoryTrustStore {
    trusted: Mutex<HashSet<PathBuf>>,
    fail: bool,
}

impl MemoryTrustStore {
    /// Un almacen que falla en cada llamada.
    #[must_use]
    pub fn failing() -> Self {
        Self {
            trusted: Mutex::default(),
            fail: true,
        }
    }

    fn check(&self) -> Result<(), TrustStoreError> {
        if self.fail {
            return Err(TrustStoreError("the fake store always fails".to_owned()));
        }
        Ok(())
    }
}

#[async_trait]
impl TrustStore for MemoryTrustStore {
    async fn is_trusted(&self, root: &RepoRoot) -> Result<bool, TrustStoreError> {
        self.check()?;
        Ok(lock(&self.trusted).contains(root.as_path()))
    }

    async fn trust(&self, root: &RepoRoot) -> Result<(), TrustStoreError> {
        self.check()?;
        lock(&self.trusted).insert(root.as_path().to_path_buf());
        Ok(())
    }
}

/// Devuelve siempre las mismas carpetas y apunta lo que se le pidio.
#[derive(Debug, Default)]
pub struct FakeScanner {
    found: Vec<PathBuf>,
    asked: Mutex<Vec<(PathBuf, usize)>>,
}

impl FakeScanner {
    /// Un buscador que encuentra `found`.
    #[must_use]
    pub fn finding(found: Vec<PathBuf>) -> Self {
        Self {
            found,
            asked: Mutex::default(),
        }
    }

    /// Carpetas y profundidades pedidas.
    #[must_use]
    pub fn asked(&self) -> Vec<(PathBuf, usize)> {
        lock(&self.asked).clone()
    }
}

#[async_trait]
impl RepoScanner for FakeScanner {
    async fn scan(&self, dir: &Path, max_depth: usize) -> Result<Vec<PathBuf>, GitReadError> {
        lock(&self.asked).push((dir.to_path_buf(), max_depth));
        Ok(self.found.clone())
    }
}

/// Acepta cualquier ruta que acabe en `git` y rechaza las demas.
#[derive(Debug, Default)]
pub struct FakeGitSetup {
    asked: Mutex<Vec<Option<PathBuf>>>,
}

impl FakeGitSetup {
    /// Rutas pedidas, en orden.
    #[must_use]
    pub fn asked(&self) -> Vec<Option<PathBuf>> {
        lock(&self.asked).clone()
    }
}

#[async_trait]
impl GitSetup for FakeGitSetup {
    async fn reconfigure(&self, binary: Option<PathBuf>) -> Result<GitVersionInfo, GitReadError> {
        lock(&self.asked).push(binary.clone());
        match binary {
            Some(path) if !path.ends_with("git") => Err(GitReadError::NotFound),
            Some(path) => Ok(GitVersionInfo {
                path,
                ..version_info(GitVersion::new(2, 47, 0))
            }),
            None => Ok(version_info(GitVersion::new(2, 45, 0))),
        }
    }
}
