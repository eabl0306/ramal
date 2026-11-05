//! Puerto de lectura de git (primer corte): version, descubrimiento, refs y log.
//!
//! Lecturas y escrituras van en puertos distintos a proposito: si una lectura
//! caliente pasa algun dia a `gix`, sera otra implementacion de este puerto, con
//! la del CLI como respaldo, y las escrituras no se tocaran nunca.

use std::fmt;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git_rudder_core_domain::{CommitRecord, RefName, RefSnapshot, RepoRoot};
use thiserror::Error;

/// Version de git: mayor, menor y parche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GitVersion {
    /// Version mayor.
    pub major: u32,
    /// Version menor.
    pub minor: u32,
    /// Parche.
    pub patch: u32,
}

impl GitVersion {
    /// Construye la version.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for GitVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Git minimo: 2.34 es la primera con firma de commits por SSH y la que trae
/// Ubuntu 22.04, el suelo de Linux de Git Rudder (ADR 0009).
pub const MINIMUM_GIT_VERSION: GitVersion = GitVersion::new(2, 34, 0);

/// Funciones de git que llegaron despues de la minima. Se detectan, no se suponen:
/// si faltan, el motor usa el camino alternativo o desactiva la funcion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GitCapabilities {
    /// `git merge-tree --write-tree` (2.38).
    pub merge_tree_write_tree: bool,
    /// `git rebase --update-refs` (2.38).
    pub rebase_update_refs: bool,
    /// Marcadores de conflicto `zdiff3` (2.35): las lineas comunes salen del
    /// bloque. Sin ellos se piden `diff3`.
    pub zdiff3: bool,
}

/// El git que va a usar el motor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitVersionInfo {
    /// Ruta del ejecutable.
    pub path: PathBuf,
    /// Version detectada.
    pub version: GitVersion,
    /// Funciones posteriores a la minima.
    pub capabilities: GitCapabilities,
}

impl GitVersionInfo {
    /// La version llega a la minima.
    #[must_use]
    pub fn is_supported(&self) -> bool {
        self.version >= MINIMUM_GIT_VERSION
    }
}

/// Desde donde recorre el log del grafo.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LogRevisions {
    /// Ramas, ramas remotas, tags y HEAD: las mismas refs que devuelve
    /// `read_refs`, asi cada fila cuelga de algo que se puede etiquetar.
    #[default]
    All,
    /// Solo estas refs, y HEAD si `include_head` (ocultar o aislar ramas).
    Refs {
        /// Refs completas.
        names: Vec<RefName>,
        /// Anadir HEAD (un HEAD separado no tiene ref que lo nombre).
        include_head: bool,
    },
}

/// Que commits leer para el grafo, en orden topologico.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogQuery {
    /// Commits que saltar desde el principio: los que ya tiene la cache.
    pub skip: usize,
    /// Maximo de commits; `None` para leer hasta el final.
    pub max_count: Option<usize>,
    /// Desde donde se recorre.
    pub revisions: LogRevisions,
}

impl LogQuery {
    /// Los primeros `max_count` commits de todas las refs.
    #[must_use]
    pub const fn first(max_count: usize) -> Self {
        Self {
            skip: 0,
            max_count: Some(max_count),
            revisions: LogRevisions::All,
        }
    }
}

/// Fallo de una lectura de git.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GitReadError {
    /// No hay ejecutable de git que lanzar.
    #[error("git executable not found")]
    NotFound,
    /// La ruta no esta dentro de un repositorio con arbol de trabajo.
    #[error("{} is not inside a git repository", path.display())]
    NotARepository {
        /// Ruta pedida.
        path: PathBuf,
    },
    /// El commit, la ref o el fichero pedido no existe.
    #[error("{what} does not exist")]
    Missing {
        /// Que se busco.
        what: String,
    },
    /// git fallo o su salida no se pudo leer. El detalle es para el log.
    #[error("git command failed: {detail}")]
    Failed {
        /// Que fallo, en ingles.
        detail: String,
    },
}

/// Lecturas de git que necesitan los casos de uso del primer corte.
#[async_trait]
pub trait GitReader: Send + Sync {
    /// Ejecutable y version de git.
    async fn version(&self) -> Result<GitVersionInfo, GitReadError>;

    /// Raiz del repositorio que contiene `path`.
    async fn discover(&self, path: &Path) -> Result<RepoRoot, GitReadError>;

    /// Refs y estado de HEAD.
    async fn read_refs(&self, repo: &RepoRoot) -> Result<RefSnapshot, GitReadError>;

    /// Commits de las ramas, ramas remotas, tags y HEAD en orden topologico,
    /// hijos antes que padres.
    async fn read_log(
        &self,
        repo: &RepoRoot,
        query: &LogQuery,
    ) -> Result<Vec<CommitRecord>, GitReadError>;
}
