//! Descubrimiento del binario de git y construccion de comandos seguros.

mod command;
mod runner;
pub mod version;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub use command::GitCommand;
pub use runner::{GitRunner, RunError, RunLimits, RunOutput};

/// Nombre del ejecutable segun el sistema.
const GIT_EXECUTABLE: &str = if cfg!(windows) { "git.exe" } else { "git" };

/// Donde esta git.
///
/// Orden (ADR 0009): el ajuste explicito si existe (en VS Code, el `git.path` que
/// expone la API `vscode.git`) y si no el primer `git` del `PATH`. El MinGit
/// empaquetado de escritorio en Windows es planificado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBinary;

impl GitBinary {
    /// Resuelve el ejecutable. Una ruta explicita se devuelve tal cual: si no
    /// existe, el primer comando fallara con "git no encontrado", que es lo que
    /// el usuario tiene que ver.
    #[must_use]
    pub fn resolve(explicit: Option<PathBuf>) -> Option<PathBuf> {
        explicit.or_else(|| std::env::var_os("PATH").and_then(|paths| Self::find_in(&paths)))
    }

    /// Primer `git` ejecutable en una lista de directorios con el formato de
    /// `PATH`.
    #[must_use]
    pub fn find_in(paths: &OsStr) -> Option<PathBuf> {
        std::env::split_paths(paths)
            .filter(|dir| !dir.as_os_str().is_empty())
            .map(|dir| dir.join(GIT_EXECUTABLE))
            .find(|candidate| is_executable(candidate))
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests;
