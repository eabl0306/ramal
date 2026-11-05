//! Commits tal como los necesita el grafo: el nodo (sha y padres) para la
//! disposicion y el registro completo (autor, fecha, asunto) para pintar la fila.

use crate::diff::FileStatus;
use crate::sha::{Sha, ShortSha};
use crate::time::GitTime;

/// Un commit como nodo del grafo: su sha y sus padres en el orden de git (el
/// primero es el primer padre).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    /// Nombre del commit.
    pub sha: Sha,
    /// Padres en orden; vacio en un commit raiz.
    pub parents: Vec<Sha>,
}

impl CommitNode {
    /// Construye el nodo.
    #[must_use]
    pub const fn new(sha: Sha, parents: Vec<Sha>) -> Self {
        Self { sha, parents }
    }

    /// Un merge tiene dos padres o mas.
    #[must_use]
    pub const fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }
}

/// Autor de un commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// Nombre tal como lo guarda git, sin mailmap.
    pub name: String,
    /// Correo tal como lo guarda git.
    pub email: String,
}

/// Lo que ensena una fila del grafo de un commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRecord {
    /// Sha y padres.
    pub node: CommitNode,
    /// Forma corta que dio git (`%h`).
    pub short_sha: ShortSha,
    /// Autor.
    pub author: Signature,
    /// Fecha de autor, con su desplazamiento.
    pub authored_at: GitTime,
    /// Primera linea del mensaje (`%s`).
    pub summary: String,
}

/// Un fichero que cambio en un commit, con sus lineas anadidas y quitadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    /// Ruta en el commit.
    pub path: String,
    /// Ruta en el padre si se renombro o se copio.
    pub old_path: Option<String>,
    /// Que le paso.
    pub status: FileStatus,
    /// Lineas anadidas y quitadas; `None` si es binario.
    pub lines: Option<LineCounts>,
}

/// Lineas anadidas y quitadas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LineCounts {
    /// Anadidas.
    pub additions: u32,
    /// Quitadas.
    pub deletions: u32,
}

/// Todo lo que ensena el panel de detalle de un commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetail {
    /// Sha completo, para copiarlo.
    pub sha: Sha,
    /// Forma corta que dio git.
    pub short_sha: ShortSha,
    /// Padres en orden: un merge los lista todos.
    pub parents: Vec<Sha>,
    /// Autor.
    pub author: Signature,
    /// Fecha de autor.
    pub authored_at: GitTime,
    /// Quien hizo el commit.
    pub committer: Signature,
    /// Fecha del commit.
    pub committed_at: GitTime,
    /// Mensaje completo, sin el salto final.
    pub message: String,
    /// Ficheros cambiados contra el primer padre (contra nada si es raiz).
    pub files: Vec<ChangedFile>,
}

impl CommitDetail {
    /// Primera linea del mensaje.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.message.lines().next().unwrap_or_default()
    }
}
