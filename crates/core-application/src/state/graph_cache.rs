//! Cache de la disposicion del grafo por repositorio, refs y filtro.
//!
//! Sin cache, cada pagina leia el log desde la fila 0 y lo disponia entero: la
//! pagina `n` costaba `O(n)` y bajar por un historial grande se notaba cada vez
//! mas. Con ella, la ventana siguiente solo lee y dispone lo que falta.
//!
//! La entrada se reconoce por la huella de las refs (que incluye HEAD y la rama
//! por defecto) y por el filtro: cualquier commit, fetch o rebase cambia la
//! huella y la entrada vieja deja de servir sin que nadie la borre. Hay una
//! entrada por repositorio, la del ultimo filtro pedido.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use git_rudder_core_domain::graph::{GraphFilter, GraphWindow, LaidOutHistory, WindowRange};
use git_rudder_core_domain::{CommitRecord, RefSnapshot, RepoId, Sha};

/// Que historia se busca: repositorio, huella de sus refs y filtro.
#[derive(Debug, Clone, Copy)]
pub struct GraphKey<'a> {
    /// Repositorio.
    pub repo_id: &'a RepoId,
    /// Huella de las refs con las que se lee.
    pub fingerprint: &'a str,
    /// Filtro.
    pub filter: &'a GraphFilter,
}

/// Cuanto hay leido de una historia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Commits leidos.
    pub len: usize,
    /// Ya no hay mas.
    pub complete: bool,
}

#[derive(Debug)]
struct Entry {
    fingerprint: String,
    filter: GraphFilter,
    history: LaidOutHistory,
}

impl Entry {
    fn matches(&self, key: &GraphKey<'_>) -> bool {
        self.fingerprint == key.fingerprint && &self.filter == key.filter
    }
}

/// Cache compartida por los casos de uso del grafo. Segura entre hilos.
#[derive(Debug, Default)]
pub struct GraphCache {
    entries: Mutex<HashMap<RepoId, Entry>>,
}

impl GraphCache {
    /// Cache vacia.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cuanto hay leido para `key`, o `None` si no hay entrada que sirva.
    #[must_use]
    pub fn progress(&self, key: &GraphKey<'_>) -> Option<Progress> {
        self.lock()
            .get(key.repo_id)
            .filter(|entry| entry.matches(key))
            .map(|entry| Progress {
                len: entry.history.len(),
                complete: entry.history.is_complete(),
            })
    }

    /// Empieza una historia vacia para `key`, con las puntas fijadas dadas.
    pub fn reset(&self, key: &GraphKey<'_>, pinned: &[Sha]) {
        self.lock().insert(
            key.repo_id.clone(),
            Entry {
                fingerprint: key.fingerprint.to_owned(),
                filter: key.filter.clone(),
                history: LaidOutHistory::new(pinned),
            },
        );
    }

    /// Anade commits si la entrada sigue siendo la de `key` y tiene `expected`
    /// commits. `false` si otra peticion se adelanto: quien llama vuelve a mirar.
    pub fn extend(
        &self,
        key: &GraphKey<'_>,
        expected: usize,
        commits: Vec<CommitRecord>,
        complete: bool,
    ) -> bool {
        self.lock()
            .get_mut(key.repo_id)
            .filter(|entry| entry.matches(key) && entry.history.len() == expected)
            .map(|entry| entry.history.extend(commits, complete))
            .is_some()
    }

    /// Ventana `range` de la historia de `key`, si hay entrada.
    #[must_use]
    pub fn window(
        &self,
        key: &GraphKey<'_>,
        range: WindowRange,
        refs: &RefSnapshot,
    ) -> Option<GraphWindow> {
        self.lock()
            .get(key.repo_id)
            .filter(|entry| entry.matches(key))
            .map(|entry| entry.history.window(range, refs))
    }

    /// Fila de un commit en la historia de `key`, si ya se leyo.
    #[must_use]
    pub fn row_of(&self, key: &GraphKey<'_>, sha: Sha) -> Option<usize> {
        self.lock()
            .get(key.repo_id)
            .filter(|entry| entry.matches(key))
            .and_then(|entry| entry.history.row_of(sha))
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<RepoId, Entry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
