//! Blame: autor, commit y fecha de cada linea de un fichero.
//!
//! Los datos de cada commit se guardan una vez y las lineas solo llevan su sha,
//! como hace `git blame --porcelain`: en un fichero de diez mil lineas con veinte
//! commits, repetir autor y asunto por linea multiplicaria la respuesta.

use crate::commit::Signature;
use crate::sha::Sha;
use crate::time::GitTime;

/// Un commit que aparece en el blame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameCommit {
    /// Sha. Las lineas sin commit (cambios del arbol de trabajo) llevan el sha cero.
    pub sha: Sha,
    /// Autor.
    pub author: Signature,
    /// Fecha de autor.
    pub authored_at: GitTime,
    /// Asunto del mensaje.
    pub summary: String,
    /// Ruta del fichero en ese commit (cambia si se renombro despues).
    pub path: String,
}

impl BlameCommit {
    /// Es el pseudo commit de los cambios que aun no tienen commit.
    #[must_use]
    pub fn is_uncommitted(&self) -> bool {
        self.sha.is_zero()
    }
}

/// Una linea del fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameLine {
    /// Numero de linea en el fichero, empezando en 1.
    pub line: u32,
    /// Numero de linea en el commit que la introdujo.
    pub original_line: u32,
    /// Commit que la introdujo.
    pub sha: Sha,
    /// Texto de la linea.
    pub text: String,
}

/// Blame de un fichero.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Blame {
    /// Lineas en orden.
    pub lines: Vec<BlameLine>,
    /// Commits que aparecen, cada uno una vez y en orden de primera aparicion.
    pub commits: Vec<BlameCommit>,
}
