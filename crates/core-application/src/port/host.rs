//! Puertos que dependen de donde corre el motor: notificar a la interfaz,
//! vigilar el disco, recordar la confianza, buscar repositorios en carpetas y
//! cambiar el ejecutable de git.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git_rudder_core_domain::{RepoId, RepoRoot};
use thiserror::Error;

use super::git_reader::{GitReadError, GitVersionInfo};

/// Algo que el motor cuenta sin que la interfaz pregunte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    /// Cambio algo en el arbol de trabajo o en `.git` de un repositorio abierto:
    /// la interfaz vuelve a leer lo que ensena.
    RepoChanged {
        /// Repositorio.
        repo_id: RepoId,
    },
}

/// Donde se entregan los eventos. Lo implementa cada host: un `Channel` de
/// Tauri en escritorio, un marco por stdout en el sidecar.
pub trait EventSink: Send + Sync {
    /// Entrega un evento. No espera respuesta y no puede fallar: si el host ya
    /// no escucha, se pierde.
    fn publish(&self, event: EngineEvent);
}

/// No se pudo vigilar un repositorio.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("could not watch the repository: {0}")]
pub struct WatchError(pub String);

/// Vigila el arbol de trabajo y `.git` de un repositorio.
pub trait RepoWatcher: Send + Sync {
    /// Empieza a vigilar `root` y publica [`EngineEvent::RepoChanged`] en
    /// `sink`, agrupando rafagas. Vigilar dos veces el mismo id no hace nada.
    ///
    /// # Errors
    ///
    /// [`WatchError`] si el sistema no deja vigilar la carpeta.
    fn watch(&self, repo_id: &RepoId, root: &RepoRoot) -> Result<(), WatchError>;
}

/// No se pudo leer o guardar la confianza.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("could not use the trust store: {0}")]
pub struct TrustStoreError(pub String);

/// Repositorios en los que el usuario confio en este dispositivo. No se
/// sincroniza: confiar es una decision sobre una carpeta de un equipo concreto.
#[async_trait]
pub trait TrustStore: Send + Sync {
    /// El usuario confio en esta raiz.
    async fn is_trusted(&self, root: &RepoRoot) -> Result<bool, TrustStoreError>;

    /// Recuerda que el usuario confia en esta raiz.
    async fn trust(&self, root: &RepoRoot) -> Result<(), TrustStoreError>;
}

/// Busca repositorios dentro de una carpeta.
#[async_trait]
pub trait RepoScanner: Send + Sync {
    /// Raices de los repositorios (y submodulos) bajo `dir`, como mucho a
    /// `max_depth` carpetas de profundidad, sin repetir.
    async fn scan(&self, dir: &Path, max_depth: usize) -> Result<Vec<PathBuf>, GitReadError>;
}

/// Cambia el ejecutable de git en caliente.
#[async_trait]
pub trait GitSetup: Send + Sync {
    /// Usa `binary` (o el del `PATH` si es `None`) y vuelve a comprobar version y
    /// capacidades.
    async fn reconfigure(&self, binary: Option<PathBuf>) -> Result<GitVersionInfo, GitReadError>;
}
