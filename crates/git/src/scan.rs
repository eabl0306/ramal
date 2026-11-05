//! `FsScanner`: busca repositorios dentro de una carpeta.
//!
//! La bienvenida ofrece anadir de golpe los repositorios que el usuario ya tiene
//! (D-01). Una carpeta es un repositorio si tiene `.git` (carpeta, o fichero en un
//! submodulo o un worktree). Se sigue bajando dentro de cada repositorio para
//! encontrar sus submodulos y los repositorios anidados, sin entrar nunca en
//! `.git` ni seguir enlaces simbolicos (un enlace a una carpeta de arriba haria
//! un ciclo). Las rutas se devuelven canonicas, asi un repositorio no sale dos
//! veces por dos caminos. Cancelar la peticion suelta el futuro y la busqueda se
//! para en la carpeta en la que estuviera.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git_rudder_core_application::{GitReadError, RepoScanner};

/// Carpetas que nunca tienen un repositorio propio y pueden tener cientos de
/// miles de entradas.
const SKIPPED_DIRS: &[&str] = &[".git", "node_modules", ".Trash", "$RECYCLE.BIN"];

/// Buscador sobre el sistema de ficheros.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsScanner;

#[async_trait]
impl RepoScanner for FsScanner {
    async fn scan(&self, dir: &Path, max_depth: usize) -> Result<Vec<PathBuf>, GitReadError> {
        let start = tokio::fs::canonicalize(dir)
            .await
            .map_err(|_| GitReadError::Missing {
                what: format!("folder {}", dir.display()),
            })?;
        let mut found = Vec::new();
        let mut seen = HashSet::new();
        let mut level = vec![start];
        for depth in 0..=max_depth {
            let mut next = Vec::new();
            for folder in level {
                if !seen.insert(folder.clone()) {
                    continue;
                }
                if is_repository(&folder).await {
                    found.push(folder.clone());
                }
                if depth < max_depth {
                    next.extend(subfolders(&folder).await);
                }
            }
            level = next;
        }
        Ok(found)
    }
}

async fn is_repository(folder: &Path) -> bool {
    tokio::fs::symlink_metadata(folder.join(".git"))
        .await
        .is_ok_and(|meta| meta.is_dir() || meta.is_file())
}

/// Subcarpetas que no son enlaces ni carpetas que se saltan.
async fn subfolders(folder: &Path) -> Vec<PathBuf> {
    let Ok(mut entries) = tokio::fs::read_dir(folder).await else {
        return Vec::new();
    };
    let mut out = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let skipped = entry
            .file_name()
            .to_str()
            .is_some_and(|name| SKIPPED_DIRS.contains(&name));
        let is_dir = entry.file_type().await.is_ok_and(|kind| kind.is_dir());
        if is_dir && !skipped {
            out.push(entry.path());
        }
    }
    out.sort();
    out
}
