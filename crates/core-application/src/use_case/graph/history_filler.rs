//! Lee de git lo que le falta a la cache del grafo, por trozos que crecen.
//!
//! Cada lectura pide al menos tantos commits como ya hay (la cache se duplica),
//! asi bajar hasta la fila `n` cuesta `O(n)` en total y no `O(n^2)`. Con un filtro
//! de autor o de fecha se lee la historia entera de una vez, porque no se sabe
//! cuantas filas quedaran despues de filtrar.

use std::sync::Arc;

use git_rudder_core_domain::graph::{GraphFilter, RefSelection, WindowRange, filter_history};
use git_rudder_core_domain::{CommitRecord, RefSnapshot, RepoRoot, Sha};

use crate::error::EngineError;
use crate::port::git_reader::{GitReader, LogQuery, LogRevisions};
use crate::state::graph_cache::{GraphCache, GraphKey, Progress};

/// Trozo minimo. El recorrido topologico de git cuesta lo mismo pida cuantos pida
/// (en cien mil commits, unos 400 ms), y dos mil commits se leen y se disponen en
/// milisegundos: asi las diez primeras paginas salen de la cache.
const MIN_BATCH: usize = 2000;

/// Vueltas maximas si otras peticiones se adelantan una y otra vez (las refs
/// cambiando a la vez que se lee). Despues se sirve lo que haya.
const MAX_ROUNDS: usize = 64;

/// Hasta donde leer.
#[derive(Debug, Clone, Copy)]
pub(super) enum Until {
    /// Lo bastante para servir la ventana con su commit de comprobacion.
    Covers(WindowRange),
    /// Hasta el final.
    Complete,
}

impl Until {
    const fn reached(self, progress: Progress) -> bool {
        match self {
            Self::Covers(range) => progress.complete || progress.len > range.end(),
            Self::Complete => progress.complete,
        }
    }

    const fn wanted(self) -> Option<usize> {
        match self {
            Self::Covers(range) => Some(range.commits_to_read()),
            Self::Complete => None,
        }
    }
}

/// La historia que se esta llenando: donde se lee, con que refs y su clave.
#[derive(Debug, Clone, Copy)]
pub(super) struct Target<'a> {
    pub(super) root: &'a RepoRoot,
    pub(super) refs: &'a RefSnapshot,
    pub(super) key: &'a GraphKey<'a>,
}

/// Llena la cache desde git.
pub(super) struct HistoryFiller {
    pub(super) git: Arc<dyn GitReader>,
    pub(super) cache: Arc<GraphCache>,
}

impl HistoryFiller {
    /// Lee hasta `until` la historia de `target`.
    pub(super) async fn fill(&self, target: Target<'_>, until: Until) -> Result<(), EngineError> {
        for _ in 0..MAX_ROUNDS {
            let Some(progress) = self.cache.progress(target.key) else {
                self.start(target).await?;
                continue;
            };
            if until.reached(progress) {
                return Ok(());
            }
            self.read_more(target, progress, until).await?;
        }
        Ok(())
    }

    async fn start(&self, target: Target<'_>) -> Result<(), EngineError> {
        let Target { root, refs, key } = target;
        // Un repositorio sin commits no tiene log que leer (y git log falla en
        // algunas versiones si HEAD no ha nacido).
        if refs.is_empty() {
            self.cache.reset(key, &[]);
            self.cache.extend(key, 0, Vec::new(), true);
            return Ok(());
        }
        if !key.filter.filters_rows() {
            self.cache.reset(key, &pinned(refs, key.filter, None));
            return Ok(());
        }
        let all = self
            .git
            .read_log(root, &query(refs, key.filter, 0, None))
            .await?;
        let kept = filter_history(all, key.filter);
        self.cache
            .reset(key, &pinned(refs, key.filter, Some(&kept)));
        self.cache.extend(key, 0, kept, true);
        Ok(())
    }

    async fn read_more(
        &self,
        target: Target<'_>,
        progress: Progress,
        until: Until,
    ) -> Result<(), EngineError> {
        let Target { root, refs, key } = target;
        let max_count = until.wanted().map(|wanted| {
            wanted
                .saturating_sub(progress.len)
                .max(progress.len)
                .max(MIN_BATCH)
        });
        let commits = self
            .git
            .read_log(root, &query(refs, key.filter, progress.len, max_count))
            .await?;
        let complete = max_count.is_none_or(|max| commits.len() < max);
        // Si otra peticion se adelanto, lo leido se tira y se vuelve a mirar.
        self.cache.extend(key, progress.len, commits, complete);
        Ok(())
    }
}

fn query(refs: &RefSnapshot, filter: &GraphFilter, skip: usize, max: Option<usize>) -> LogQuery {
    LogQuery {
        skip,
        max_count: max,
        revisions: revisions(refs, filter),
    }
}

/// Ocultar recorre todas las refs menos las ocultas, y HEAD salvo que su rama
/// este oculta. Aislar recorre solo las aisladas.
fn revisions(refs: &RefSnapshot, filter: &GraphFilter) -> LogRevisions {
    match filter.selection() {
        RefSelection::All => LogRevisions::All,
        RefSelection::Only(names) => LogRevisions::Refs {
            names,
            include_head: false,
        },
        RefSelection::Excluding(hidden) => LogRevisions::Refs {
            names: refs
                .refs()
                .iter()
                .map(|git_ref| git_ref.name.clone())
                .filter(|name| !hidden.contains(name))
                .collect(),
            include_head: refs.head_branch().is_none_or(|head| !hidden.contains(head)),
        },
    }
}

/// La rama por defecto se fija a la izquierda solo si se ve: si el filtro la
/// oculta o la deja fuera, su carril reservado quedaria vacio para siempre.
fn pinned(refs: &RefSnapshot, filter: &GraphFilter, kept: Option<&[CommitRecord]>) -> Vec<Sha> {
    let Some(tip) = refs.default_branch_tip() else {
        return Vec::new();
    };
    let points_at_tip = |wanted: &dyn Fn(&git_rudder_core_domain::RefName) -> bool| {
        refs.refs()
            .iter()
            .any(|git_ref| git_ref.target == tip && wanted(&git_ref.name))
    };
    let selected = match filter.selection() {
        RefSelection::All => true,
        RefSelection::Only(names) => points_at_tip(&|name| names.contains(name)),
        RefSelection::Excluding(names) => points_at_tip(&|name| !names.contains(name)),
    };
    let visible = kept.is_none_or(|kept| kept.iter().any(|commit| commit.node.sha == tip));
    if selected && visible {
        vec![tip]
    } else {
        Vec::new()
    }
}
