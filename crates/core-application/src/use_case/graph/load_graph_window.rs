//! `LoadGraphWindow`: log + refs + disposicion + ventana.
//!
//! Lee desde la fila 0 hasta el final de la ventana mas un commit (que solo dice
//! si hay mas), dispone todo y recorta. Leer desde el principio cuesta O(offset),
//! pero garantiza que los carriles de una fila no cambian segun la ventana que se
//! pida. La cache de disposicion por HEAD esta planificada.
//!
//! Cada llamada lee el repositorio tal como esta en ese momento, asi que entre
//! una pagina y la siguiente puede haber un commit, un fetch o un rebase. Por eso
//! cada ventana lleva la huella de las refs con las que se leyo, y la pagina
//! siguiente se pide con esa huella: si ya no coincide, se responde
//! [`EngineError::StaleWindow`] sin leer el log.
//!
//! Queda un hueco de milisegundos: si las refs cambian entre `read_refs` y
//! `read_log` de una misma llamada, esa pagina puede salir del historial nuevo
//! con la huella vieja. La peticion siguiente ya ve la huella cambiada y la
//! interfaz vuelve a empezar. Cerrarlo del todo pediria pasar a git los shas de
//! la foto por stdin en vez de las refs, y no compensa por ese hueco.

use std::sync::Arc;

use git_rudder_core_domain::RepoId;
use git_rudder_core_domain::graph::{GraphWindow, WindowRange, graph_window};

use crate::error::EngineError;
use crate::port::git_reader::{GitReader, LogQuery};
use crate::state::open_repositories::OpenRepositories;

/// Carga una ventana del grafo de un repositorio abierto.
pub struct LoadGraphWindow {
    git: Arc<dyn GitReader>,
    registry: Arc<OpenRepositories>,
}

impl std::fmt::Debug for LoadGraphWindow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoadGraphWindow")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl LoadGraphWindow {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(git: Arc<dyn GitReader>, registry: Arc<OpenRepositories>) -> Self {
        Self { git, registry }
    }

    /// Ventana `[offset, offset + limit)` del grafo de `repo_id`. `snapshot` es
    /// la huella de una ventana anterior (`GraphWindow::snapshot`); sin ella no
    /// se comprueba nada, que es lo que pide la primera pagina.
    ///
    /// # Errors
    ///
    /// [`EngineError::InvalidWindow`] si `limit` esta fuera de `1..=1000`,
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto,
    /// [`EngineError::StaleWindow`] si `snapshot` ya no es la huella de las refs
    /// y [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        offset: usize,
        limit: usize,
        snapshot: Option<&str>,
    ) -> Result<GraphWindow, EngineError> {
        let range = WindowRange::new(offset, limit)?;
        let root = self
            .registry
            .get(repo_id)
            .ok_or_else(|| EngineError::RepositoryNotOpen {
                repo_id: repo_id.clone(),
            })?;
        let refs = self.git.read_refs(&root).await?;
        if snapshot.is_some_and(|expected| expected != refs.fingerprint()) {
            return Err(EngineError::StaleWindow);
        }
        // Un repositorio sin commits no tiene log que leer (y git log falla en
        // algunas versiones si HEAD no ha nacido).
        let commits = if refs.is_empty() {
            Vec::new()
        } else {
            let query = LogQuery {
                max_count: range.commits_to_read(),
            };
            self.git.read_log(&root, &query).await?
        };
        Ok(graph_window(range, commits, &refs))
    }
}

#[cfg(test)]
mod tests;
