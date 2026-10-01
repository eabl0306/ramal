//! Commits tal como los necesita el grafo: el nodo (sha y padres) para la
//! disposicion y el registro completo (autor, fecha, asunto) para pintar la fila.

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
