//! `LoadGraphWindow`: refs, cache de disposicion y ventana.
//!
//! La disposicion se guarda por repositorio, refs y filtro ([`GraphCache`]):
//! pedir la ventana siguiente solo lee y dispone lo que falta. Los carriles de
//! una fila no cambian segun la ventana que se pida, porque el calculo es
//! incremental por filas.
//!
//! Cada llamada lee las refs tal como estan en ese momento, asi que entre una
//! pagina y la siguiente puede haber un commit, un fetch o un rebase. Por eso
//! cada ventana lleva la huella de las refs con las que se leyo, y la pagina
//! siguiente se pide con esa huella: si ya no coincide, se responde
//! [`EngineError::StaleWindow`] sin leer el log. La huella tambien es la clave de
//! la cache, asi que una entrada vieja deja de servir sola.
//!
//! Queda un hueco de milisegundos: si las refs cambian entre `read_refs` y
//! `read_log` de una misma llamada, esa pagina puede salir del historial nuevo
//! con la huella vieja. La peticion siguiente ya ve la huella cambiada y la
//! interfaz vuelve a empezar. Cerrarlo del todo pediria pasar a git los shas de
//! la foto por stdin en vez de las refs, y no compensa por ese hueco.

use std::sync::Arc;

use git_rudder_core_domain::RepoId;
use git_rudder_core_domain::graph::{GraphFilter, GraphWindow, WindowRange};

use super::history_filler::{HistoryFiller, Target, Until};
use crate::error::EngineError;
use crate::port::git_reader::GitReader;
use crate::state::graph_cache::{GraphCache, GraphKey};
use crate::state::open_repositories::OpenRepositories;

/// Que ventana se pide.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowRequest {
    /// Primera fila.
    pub offset: usize,
    /// Filas, de 1 a 1000.
    pub limit: usize,
    /// Huella de una ventana anterior (`GraphWindow::snapshot`); sin ella no se
    /// comprueba nada, que es lo que pide la primera pagina.
    pub snapshot: Option<String>,
    /// Ramas ocultas o aisladas, autor y fechas.
    pub filter: GraphFilter,
}

/// Carga una ventana del grafo de un repositorio abierto.
pub struct LoadGraphWindow {
    filler: HistoryFiller,
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
    pub fn new(
        git: Arc<dyn GitReader>,
        registry: Arc<OpenRepositories>,
        cache: Arc<GraphCache>,
    ) -> Self {
        Self {
            filler: HistoryFiller { git, cache },
            registry,
        }
    }

    /// Ventana `[offset, offset + limit)` del grafo de `repo_id`.
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
        request: &WindowRequest,
    ) -> Result<GraphWindow, EngineError> {
        let range = WindowRange::new(request.offset, request.limit)?;
        let repo = self.registry.require(repo_id)?;
        let refs = self.filler.git.read_refs(&repo.root).await?;
        let fingerprint = refs.fingerprint();
        if request
            .snapshot
            .as_deref()
            .is_some_and(|expected| expected != fingerprint)
        {
            return Err(EngineError::StaleWindow);
        }
        let key = GraphKey {
            repo_id,
            fingerprint: &fingerprint,
            filter: &request.filter,
        };
        let target = Target {
            root: &repo.root,
            refs: &refs,
            key: &key,
        };
        self.filler.fill(target, Until::Covers(range)).await?;
        self.filler
            .cache
            .window(&key, range, &refs)
            .ok_or(EngineError::StaleWindow)
    }
}

#[cfg(test)]
mod tests;
