//! Identidad de un repositorio abierto: su raiz en disco y el id opaco que ve la
//! interfaz.
//!
//! La interfaz nunca maneja rutas: habla con el motor por `RepoId`, y solo el
//! motor sabe a que raiz corresponde. Por eso un `repoId` inventado no da acceso a
//! nada (se busca en el registro y falla).

use std::fmt;
use std::path::{Path, PathBuf};

/// Raiz del arbol de trabajo de un repositorio, tal como la devuelve git
/// (`rev-parse --show-toplevel`).
///
/// El dominio no abre la ruta ni comprueba que exista: solo la transporta.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoRoot(PathBuf);

impl RepoRoot {
    /// Envuelve la ruta de la raiz.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// La ruta de la raiz.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Identificador opaco de un repositorio abierto en esta sesion del motor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RepoId(String);

impl RepoId {
    /// Envuelve un identificador. No se valida: es opaco y solo sirve para buscar
    /// en el registro de repositorios abiertos.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// El texto del identificador.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RepoId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
