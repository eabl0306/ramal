//! `SearchHistory`: buscar por mensaje, sha o autor en todo el historial.
//!
//! La busqueda del prototipo solo miraba las filas cargadas. Aqui la hace git
//! sobre las mismas refs que el grafo, y cada coincidencia lleva su fila para que
//! la interfaz salte a ella aunque no este cargada: si hace falta, la cache del
//! grafo se completa hasta el final. Cancelar la peticion para a git.

use std::sync::Arc;

use git_rudder_core_domain::RepoId;
use git_rudder_core_domain::graph::GraphFilter;
use git_rudder_core_domain::history::{SearchHit, SearchQuery};

use super::history_filler::{HistoryFiller, Target, Until};
use crate::error::EngineError;
use crate::port::git_reader::GitReader;
use crate::port::repo_reader::HistoryReader;
use crate::state::graph_cache::{GraphCache, GraphKey};
use crate::state::open_repositories::OpenRepositories;

/// Coincidencias como mucho por busqueda.
pub const MAX_SEARCH_RESULTS: usize = 500;

/// Busca en el historial de un repositorio abierto.
pub struct SearchHistory {
    filler: HistoryFiller,
    history: Arc<dyn HistoryReader>,
    registry: Arc<OpenRepositories>,
}

impl std::fmt::Debug for SearchHistory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SearchHistory")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl SearchHistory {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(
        git: Arc<dyn GitReader>,
        history: Arc<dyn HistoryReader>,
        registry: Arc<OpenRepositories>,
        cache: Arc<GraphCache>,
    ) -> Self {
        Self {
            filler: HistoryFiller { git, cache },
            history,
            registry,
        }
    }

    /// Hasta `limit` commits (como mucho [`MAX_SEARCH_RESULTS`]) que coinciden
    /// con `text`, con su fila en el grafo filtrado por `filter` si sale en el.
    /// Un texto vacio no busca nada.
    ///
    /// # Errors
    ///
    /// [`EngineError::RepositoryNotOpen`] si el id no esta abierto y
    /// [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        repo_id: &RepoId,
        text: &str,
        limit: usize,
        filter: &GraphFilter,
    ) -> Result<Vec<SearchHit>, EngineError> {
        let Some(query) = SearchQuery::parse(text) else {
            return Ok(Vec::new());
        };
        let repo = self.registry.require(repo_id)?;
        let limit = limit.clamp(1, MAX_SEARCH_RESULTS);
        let found = self.history.search(&repo.root, &query, limit).await?;
        if found.is_empty() {
            return Ok(Vec::new());
        }
        let refs = self.filler.git.read_refs(&repo.root).await?;
        let fingerprint = refs.fingerprint();
        let key = GraphKey {
            repo_id,
            fingerprint: &fingerprint,
            filter,
        };
        let cache = &self.filler.cache;
        let known = |sha| cache.row_of(&key, sha).is_some();
        if !found.iter().all(|hit| known(hit.commit.node.sha)) {
            let target = Target {
                root: &repo.root,
                refs: &refs,
                key: &key,
            };
            self.filler.fill(target, Until::Complete).await?;
        }
        Ok(found
            .into_iter()
            .map(|hit| SearchHit {
                row: cache.row_of(&key, hit.commit.node.sha),
                commit: hit.commit,
                matched: hit.matched,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests;
