//! `EngineError`, el unico error que sale de un caso de uso.
//!
//! Las variantes son cerradas: `ramal-engine` traduce cada una a un `ErrorKind`
//! estable del protocolo, que es lo que la interfaz usa para decidir y traducir.

use std::path::PathBuf;

use ramal_core_domain::RepoId;
use ramal_core_domain::graph::WindowError;
use thiserror::Error;

use crate::port::git_reader::{GitReadError, GitVersion};

/// Fallo de un caso de uso del motor.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngineError {
    /// El `repoId` no esta en el registro de repositorios abiertos.
    #[error("repository {repo_id} is not open")]
    RepositoryNotOpen {
        /// Id pedido.
        repo_id: RepoId,
    },
    /// La ruta no esta dentro de un repositorio.
    #[error("{} is not inside a git repository", path.display())]
    NotARepository {
        /// Ruta pedida.
        path: PathBuf,
    },
    /// No hay git instalado o no se pudo lanzar.
    #[error("git executable not found")]
    GitUnavailable,
    /// El git instalado es anterior al minimo.
    #[error("git {found} is not supported; the minimum is {minimum}")]
    GitVersionUnsupported {
        /// Version encontrada.
        found: GitVersion,
        /// Version minima.
        minimum: GitVersion,
    },
    /// La ventana del grafo pedida no es valida.
    #[error("invalid graph window: {0}")]
    InvalidWindow(#[from] WindowError),
    /// Se pidio una pagina del grafo con la huella de unas refs que ya no son
    /// las del repositorio: la interfaz tiene que volver a empezar desde la
    /// fila 0 en vez de pegar paginas de dos historiales.
    #[error("the graph changed since the first page was read")]
    StaleWindow,
    /// git fallo. El detalle va solo al log; la interfaz recibe un tipo estable.
    #[error("git command failed: {detail}")]
    GitFailed {
        /// Que fallo, en ingles.
        detail: String,
    },
}

impl From<GitReadError> for EngineError {
    fn from(error: GitReadError) -> Self {
        match error {
            GitReadError::NotFound => Self::GitUnavailable,
            GitReadError::NotARepository { path } => Self::NotARepository { path },
            GitReadError::Failed { detail } => Self::GitFailed { detail },
        }
    }
}
